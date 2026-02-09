<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { open } from '@tauri-apps/plugin-shell';
  import { LayoutDashboard, TrendingUp, Plus, Settings as SettingsIcon, Github, Wallet, BookOpen, FileText } from 'lucide-svelte';
  import Overview from './lib/Overview.svelte';
  import Analytics from './lib/Analytics.svelte';
  import Accounts from './lib/Accounts.svelte';
  import Reports from './lib/Reports.svelte';
  import QuickEntry from './lib/QuickEntry.svelte';
  import EditTransaction from './lib/EditTransaction.svelte';
  import CommandPalette from './lib/CommandPalette.svelte';
  import CategoryManager from './lib/CategoryManager.svelte';
  import ContainerManager from './lib/ContainerManager.svelte';
  import ImportCSV from './lib/ImportCSV.svelte';
  import Settings from './lib/Settings.svelte';
  import Dropdown from './lib/Dropdown.svelte';
  import Toast from './lib/Toast.svelte';

  interface Transaction {
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
    container_id: number;
    account_id: number;
    transfer_id: number;
    transfer_account_id: number;
  }

  interface Container {
    id: number;
    name: string;
    created_at: string;
    is_default: boolean;
  }

  interface Account {
    id: number;
    name: string;
    account_type: string;
    opening_balance: number;
    container_id: number;
    created_at: string;
  }

  interface AccountBalance extends Account {
    balance: number;
  }

  let activeTab: 'overview' | 'analytics' | 'accounts' | 'reports' = 'overview';
  let showQuickEntry = false;
  let showEditTransaction = false;
  let editingTransaction: Transaction | null = null;
  let showCommandPalette = false;
  let showCategoryManager = false;
  let showContainerManager = false;
  let showImportCSV = false;
  let showSettings = false;
  let monthlyBalance = 0;
  let allTimeBalance = 0;
  let transactions: Transaction[] = [];
  let categoryTotals: Array<[string, number]> = [];
  let availableMonths: string[] = [];
  let selectedMonth: string = '';
  let containers: Container[] = [];
  let selectedContainer: Container | null = null;
  let accounts: Account[] = [];
  let accountBalances: AccountBalance[] = [];
  let toastMessage = '';
  let toastType: 'success' | 'error' | 'info' | 'warning' = 'info';
  let showToast = false;

  function getCurrentMonth() {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
  }

  async function loadContainers() {
    try {
      containers = await invoke<Container[]>('get_containers');
      if (containers.length > 0 && !selectedContainer) {
        selectedContainer = containers.find(c => c.is_default) || containers[0];
      }
    } catch (error) {
      console.error('Failed to load containers:', error);
    }
  }

  async function loadAccounts() {
    if (!selectedContainer) return;
    try {
      accounts = await invoke<Account[]>('get_accounts', { containerId: selectedContainer.id });
    } catch (error) {
      console.error('Failed to load accounts:', error);
      accounts = [];
    }
  }

  async function loadAccountBalances() {
    if (!selectedContainer) return;
    try {
      accountBalances = await invoke<AccountBalance[]>('get_account_balances', { containerId: selectedContainer.id });
    } catch (error) {
      console.error('Failed to load account balances:', error);
      accountBalances = [];
    }
  }

  async function loadAvailableMonths() {
    if (!selectedContainer) return;
    
    try {
      const months = await invoke<string[]>('get_available_months', { containerId: selectedContainer.id });
      const currentMonth = getCurrentMonth();
      
      const monthSet = new Set(months);
      monthSet.add(currentMonth);
      
      availableMonths = Array.from(monthSet).sort((a, b) => b.localeCompare(a));
      
      if (!selectedMonth) {
        selectedMonth = currentMonth;
      }
    } catch (error) {
      console.error('Failed to load months:', error);
      availableMonths = [getCurrentMonth()];
      selectedMonth = getCurrentMonth();
    }
  }

  async function loadData() {
    if (!selectedContainer) return;
    
    try {
      if (selectedMonth === getCurrentMonth()) {
        monthlyBalance = await invoke<number>('get_monthly_balance', { containerId: selectedContainer.id });
        transactions = await invoke<Transaction[]>('get_transactions', { containerId: selectedContainer.id, limit: 50 });
        categoryTotals = await invoke<Array<[string, number]>>('get_category_totals', { containerId: selectedContainer.id });
      } else {
        monthlyBalance = await invoke<number>('get_balance_for_month', { containerId: selectedContainer.id, month: selectedMonth });
        transactions = await invoke<Transaction[]>('get_transactions_for_month', { containerId: selectedContainer.id, month: selectedMonth, limit: 50 });
        categoryTotals = await invoke<Array<[string, number]>>('get_category_totals_for_month', { containerId: selectedContainer.id, month: selectedMonth });
      }
      allTimeBalance = await invoke<number>('get_all_time_balance', { containerId: selectedContainer.id });
    } catch (error) {
      console.error('Failed to load data:', error);
    }
  }

  $: if (selectedMonth) {
    loadData();
  }

  $: if (selectedContainer) {
    loadAvailableMonths();
    loadAccounts();
    loadAccountBalances();
    loadData();
  }

  async function handleAddTransaction(event: CustomEvent) {
    if (!selectedContainer) return;
    
    const { amount, description, category, accountId } = event.detail;
    try {
      await invoke('add_transaction', {
        amount: Math.round(amount * 100),
        description: description || null,
        category: category || null,
        accountId,
        containerId: selectedContainer.id,
      });
      await loadData();
      await loadAccountBalances();
      showQuickEntry = false;
    } catch (error) {
      console.error('Failed to add transaction:', error);
    }
  }

  async function handleAddTransfer(event: CustomEvent) {
    if (!selectedContainer) return;

    const { amount, description, fromAccountId, toAccountId } = event.detail;
    try {
      await invoke('add_transfer', {
        amount: Math.round(amount * 100),
        description: description || null,
        fromAccountId,
        toAccountId,
        containerId: selectedContainer.id,
      });
      await loadData();
      await loadAccountBalances();
      showQuickEntry = false;
    } catch (error) {
      console.error('Failed to add transfer:', error);
    }
  }

  async function handleDeleteTransaction(event: CustomEvent) {
    try {
      await invoke('delete_transaction', { id: event.detail.id });
      await loadData();
      await loadAccountBalances();
    } catch (error) {
      console.error('Failed to delete transaction:', error);
    }
  }

  function handleEditTransaction(event: CustomEvent) {
    if (event.detail.transaction?.transfer_id) {
      return;
    }
    editingTransaction = event.detail.transaction;
    showEditTransaction = true;
  }

  async function handleUpdateTransaction(event: CustomEvent) {
    const { id, amount, description, category, accountId } = event.detail;
    try {
      await invoke('update_transaction', {
        id,
        amount,
        description,
        category,
        accountId,
      });
      await loadData();
      await loadAccountBalances();
      showEditTransaction = false;
      editingTransaction = null;
    } catch (error) {
      console.error('Failed to update transaction:', error);
    }
  }

  async function handleExport() {
    if (!selectedContainer) return;
    
    try {
      const month = selectedMonth || getCurrentMonth();
      const reports = await invoke<{ profit_loss: string; balance_sheet: string; transactions: string }>('export_reports_csv', {
        containerId: selectedContainer.id,
        month,
      });
      const profitLossPath = await save({
        defaultPath: `spent-${selectedContainer.name}-${month}-laba-rugi.csv`,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });

      if (!profitLossPath) {
        return;
      }

      const balanceSheetPath = await save({
        defaultPath: `spent-${selectedContainer.name}-${month}-posisi-keuangan.csv`,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });

      if (!balanceSheetPath) {
        return;
      }

      const transactionsPath = await save({
        defaultPath: `spent-${selectedContainer.name}-${month}-transaksi.csv`,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });

      if (!transactionsPath) {
        return;
      }

      await writeTextFile(profitLossPath, reports.profit_loss);
      await writeTextFile(balanceSheetPath, reports.balance_sheet);
      await writeTextFile(transactionsPath, reports.transactions);

      toastMessage =
        `CSV exported successfully:\n` +
        `${profitLossPath}\n` +
        `${balanceSheetPath}\n` +
        `${transactionsPath}`;
      toastType = 'success';
      showToast = true;
    } catch (error) {
      console.error('Export failed:', error);
      toastMessage = 'Failed to export CSV. Please try again.';
      toastType = 'error';
      showToast = true;
    }
  }

  async function openGitHub() {
    try {
      await open('https://github.com/FrogSnot/Spent');
    } catch (error) {
      console.error('Failed to open GitHub:', error);
    }
  }

  function handleCommand(event: CustomEvent) {
    const { action } = event.detail;
    showCommandPalette = false;
    
    switch (action) {
      case 'add':
        showQuickEntry = true;
        break;
      case 'categories':
        showCategoryManager = true;
        break;
      case 'containers':
        showContainerManager = true;
        break;
      case 'import':
        showImportCSV = true;
        break;
      case 'settings':
        showSettings = true;
        break;
      case 'export':
        handleExport();
        break;
      case 'analytics':
        activeTab = 'analytics';
        break;
      case 'reports':
        activeTab = 'reports';
        break;
      case 'transactions':
        activeTab = 'overview';
        break;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
      event.preventDefault();
      showCommandPalette = !showCommandPalette;
    }
    if ((event.ctrlKey || event.metaKey) && event.key === 'n') {
      event.preventDefault();
      showQuickEntry = !showQuickEntry;
    }
    if (event.key === 'Escape') {
      showQuickEntry = false;
      showCommandPalette = false;
      showCategoryManager = false;
      showContainerManager = false;
      showImportCSV = false;
      showSettings = false;
    }
  }

  function handleContainerChange(event: CustomEvent) {
    const containerId = event.detail.value as number;
    selectedContainer = containers.find(c => c.id === containerId) || null;
  }

  $: containerOptions = containers.map(c => ({
    value: c.id,
    label: c.is_default ? `${c.name} (Default)` : c.name
  }));

  function handleContainerDeleted(event: CustomEvent) {
    const deletedId = event.detail.id;
    if (selectedContainer?.id === deletedId) {
      selectedContainer = containers.find(c => c.is_default) || containers[0] || null;
    }
  }

  onMount(async () => {
    await loadContainers();
    await loadAvailableMonths();
    await loadAccounts();
    await loadAccountBalances();
    await loadData();
    
    const handleKeydownEvent = (event: KeyboardEvent) => handleKeydown(event);
    window.addEventListener('keydown', handleKeydownEvent);
    
    return () => window.removeEventListener('keydown', handleKeydownEvent);
  });
</script>

<main class="min-h-screen bg-gray-950 text-gray-100 flex">
  <aside class="w-56 lg:w-64 bg-gray-900 border-r border-gray-800 flex flex-col flex-shrink-0 hidden sm:flex">
    <div class="p-6 border-b border-gray-800">
      <h1 class="text-2xl font-black text-white tracking-tight">Akuntansi</h1>
      <p class="text-xs text-gray-500 mt-1">Pembukuan UMKM</p>
    </div>

    {#if selectedContainer}
      <div class="px-4 pt-4 pb-2">
        <label class="block text-xs font-semibold text-gray-500 mb-2 uppercase tracking-wider">Active Container</label>
        <Dropdown
          value={selectedContainer.id}
          options={containerOptions}
          icon={Wallet}
          on:change={handleContainerChange}
        />
      </div>
    {/if}

    <nav class="flex-1 p-4 space-y-1">
      <button
        on:click={() => (activeTab = 'overview')}
        class="w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all {activeTab === 'overview'
          ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20'
          : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
      >
        <LayoutDashboard size={20} />
        <span class="font-medium">Overview</span>
      </button>

      <button
        on:click={() => (activeTab = 'analytics')}
        class="w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all {activeTab === 'analytics'
          ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20'
          : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
      >
        <TrendingUp size={20} />
        <span class="font-medium">Analytics</span>
      </button>

      <button
        on:click={() => (activeTab = 'accounts')}
        class="w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all {activeTab === 'accounts'
          ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20'
          : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
      >
        <BookOpen size={20} />
        <span class="font-medium">Akun</span>
      </button>

      <button
        on:click={() => (activeTab = 'reports')}
        class="w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all {activeTab === 'reports'
          ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20'
          : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
      >
        <FileText size={20} />
        <span class="font-medium">Laporan</span>
      </button>
    </nav>

    <div class="p-4 space-y-3 border-t border-gray-800">
      <button
        on:click={() => (showQuickEntry = true)}
        class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-all shadow-lg shadow-blue-600/20"
      >
        <Plus size={20} />
        Quick Add
      </button>
      
      <button
        on:click={() => (showCommandPalette = true)}
        class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg font-medium transition-all text-sm"
      >
        <SettingsIcon size={16} />
        Commands
      </button>
      
      <div class="flex items-center justify-between px-2">
        <button
          on:click={openGitHub}
          class="flex items-center gap-1.5 text-gray-500 hover:text-gray-300 transition-colors text-xs hover:underline cursor-pointer"
        >
          <Github size={14} />
          <span>v1.1.6</span>
        </button>
        <button
          on:click={handleExport}
          class="text-xs text-gray-500 hover:text-gray-300 transition-colors"
        >
          Export 3 CSV
        </button>
      </div>
    </div>
  </aside>

  <div class="flex-1 overflow-hidden flex flex-col">
    {#if activeTab === 'overview'}
      <Overview
        {monthlyBalance}
        {allTimeBalance}
        {transactions}
        {selectedMonth}
        {availableMonths}
        {accounts}
        on:delete={handleDeleteTransaction}
        on:edit={handleEditTransaction}
        on:refresh={loadData}
        on:monthChange={(e) => selectedMonth = e.detail.month}
      />
    {:else if activeTab === 'analytics'}
      <Analytics {categoryTotals} {transactions} {monthlyBalance} />
    {:else if activeTab === 'accounts'}
      <Accounts
        accounts={accountBalances}
        containerId={selectedContainer?.id}
        on:refresh={() => {
          loadAccounts();
          loadAccountBalances();
        }}
      />
    {:else if activeTab === 'reports'}
      <Reports
        containerId={selectedContainer?.id}
        {selectedMonth}
        {availableMonths}
        on:monthChange={(e) => selectedMonth = e.detail.month}
      />
    {/if}
  </div>

  {#if showQuickEntry}
    <QuickEntry
      {accounts}
      on:add={handleAddTransaction}
      on:transfer={handleAddTransfer}
      on:close={() => (showQuickEntry = false)}
    />
  {/if}

  {#if showCommandPalette}
    <CommandPalette
      on:command={handleCommand}
      on:close={() => (showCommandPalette = false)}
    />
  {/if}

  {#if showCategoryManager}
    <CategoryManager
      on:close={() => (showCategoryManager = false)}
    />
  {/if}

  {#if showContainerManager}
    <ContainerManager
      on:close={() => {
        showContainerManager = false;
        loadContainers();
      }}
      on:containerDeleted={handleContainerDeleted}
      on:containerUpdated={loadContainers}
    />
  {/if}

  {#if showEditTransaction && editingTransaction}
    <EditTransaction
      transaction={editingTransaction}
      accounts={accounts}
      on:save={handleUpdateTransaction}
      on:close={() => {
        showEditTransaction = false;
        editingTransaction = null;
      }}
    />
  {/if}

  {#if showImportCSV && selectedContainer}
    <ImportCSV
      containerId={selectedContainer.id}
      on:imported={loadData}
      on:close={() => (showImportCSV = false)}
    />
  {/if}

  {#if showSettings}
    <Settings
      on:close={() => (showSettings = false)}
    />
  {/if}

  {#if showToast}
    <Toast
      message={toastMessage}
      type={toastType}
      onClose={() => (showToast = false)}
    />
  {/if}
</main>
