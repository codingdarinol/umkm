<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { LayoutDashboard, TrendingUp, Plus, Settings, Github } from 'lucide-svelte';
  import Overview from './lib/Overview.svelte';
  import Analytics from './lib/Analytics.svelte';
  import QuickEntry from './lib/QuickEntry.svelte';
  import CommandPalette from './lib/CommandPalette.svelte';
  import CategoryManager from './lib/CategoryManager.svelte';

  interface Transaction {
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
  }

  let activeTab: 'overview' | 'analytics' = 'overview';
  let showQuickEntry = false;
  let showCommandPalette = false;
  let showCategoryManager = false;
  let monthlyBalance = 0;
  let allTimeBalance = 0;
  let transactions: Transaction[] = [];
  let categoryTotals: Array<[string, number]> = [];
  let availableMonths: string[] = [];
  let selectedMonth: string = '';

  function getCurrentMonth() {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
  }

  async function loadAvailableMonths() {
    try {
      const months = await invoke<string[]>('get_available_months');
      availableMonths = months.length > 0 ? months : [getCurrentMonth()];
      if (!selectedMonth) {
        selectedMonth = getCurrentMonth();
      }
    } catch (error) {
      console.error('Failed to load months:', error);
      availableMonths = [getCurrentMonth()];
      selectedMonth = getCurrentMonth();
    }
  }

  async function loadData() {
    try {
      if (selectedMonth === getCurrentMonth()) {
        monthlyBalance = await invoke<number>('get_monthly_balance');
        transactions = await invoke<Transaction[]>('get_transactions', { limit: 50 });
        categoryTotals = await invoke<Array<[string, number]>>('get_category_totals');
      } else {
        monthlyBalance = await invoke<number>('get_balance_for_month', { month: selectedMonth });
        transactions = await invoke<Transaction[]>('get_transactions_for_month', { month: selectedMonth, limit: 50 });
        categoryTotals = await invoke<Array<[string, number]>>('get_category_totals_for_month', { month: selectedMonth });
      }
      allTimeBalance = await invoke<number>('get_all_time_balance');
    } catch (error) {
      console.error('Failed to load data:', error);
    }
  }

  $: if (selectedMonth) {
    loadData();
  }

  async function handleAddTransaction(event: CustomEvent) {
    const { amount, description, category } = event.detail;
    try {
      await invoke('add_transaction', {
        amount: Math.round(amount * 100),
        description: description || null,
        category: category || null,
      });
      await loadData();
      showQuickEntry = false;
    } catch (error) {
      console.error('Failed to add transaction:', error);
    }
  }

  async function handleDeleteTransaction(event: CustomEvent) {
    try {
      await invoke('delete_transaction', { id: event.detail.id });
      await loadData();
    } catch (error) {
      console.error('Failed to delete transaction:', error);
    }
  }

  async function handleExport() {
    try {
      const csv = await invoke<string>('export_csv');
      const path = await save({
        defaultPath: 'spent-export.csv',
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });
      
      if (path) {
        await writeTextFile(path, csv);
      }
    } catch (error) {
      console.error('Export failed:', error);
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
      case 'export':
        handleExport();
        break;
      case 'analytics':
        activeTab = 'analytics';
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
    }
  }

  onMount(async () => {
    await loadAvailableMonths();
    await loadData();
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });
</script>

<main class="min-h-screen bg-gray-950 text-gray-100 flex">
  <aside class="w-64 bg-gray-900 border-r border-gray-800 flex flex-col">
    <div class="p-6 border-b border-gray-800">
      <h1 class="text-2xl font-black text-white tracking-tight">Spent</h1>
      <p class="text-xs text-gray-500 mt-1">Finance Tracker</p>
    </div>

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
        <Settings size={16} />
        Commands
      </button>
      
      <div class="flex items-center justify-between px-2">
        <a 
          href="https://github.com/yourusername/spent" 
          target="_blank"
          class="flex items-center gap-1.5 text-gray-500 hover:text-gray-300 transition-colors text-xs"
        >
          <Github size={14} />
          <span>v0.1.0</span>
        </a>
        <button
          on:click={handleExport}
          class="text-xs text-gray-500 hover:text-gray-300 transition-colors"
        >
          Export CSV
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
        on:delete={handleDeleteTransaction}
        on:refresh={loadData}
        on:monthChange={(e) => selectedMonth = e.detail.month}
      />
    {:else if activeTab === 'analytics'}
      <Analytics {categoryTotals} {transactions} {monthlyBalance} />
    {/if}
  </div>

  {#if showQuickEntry}
    <QuickEntry
      on:add={handleAddTransaction}
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
</main>
