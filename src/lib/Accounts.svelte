<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly } from 'svelte/transition';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { Plus, X, BookOpen } from 'lucide-svelte';
  import Dropdown from './Dropdown.svelte';
  import { currencySettings, formatCurrency as formatCurrencyHelper } from './stores';

  export let accounts: Array<{
    id: number;
    name: string;
    account_type: string;
    opening_balance: number;
    balance: number;
    container_id: number;
    created_at: string;
  }> = [];
  export let containerId: number | null = null;

  const dispatch = createEventDispatcher();

  interface AccountTransaction {
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
    account_id: number;
    transfer_id: number;
    transfer_account_id: number;
  }

  let showAddAccount = false;
  let showDrawer = false;
  let selectedAccount: typeof accounts[number] | null = null;
  let accountTransactions: AccountTransaction[] = [];
  let isLoadingTransactions = false;
  let name = '';
  let accountType: 'asset' | 'contra_asset' | 'liability' | 'equity' | '' = '';
  let openingBalance = '';
  let isSaving = false;

  const typeOptions = [
    { value: 'asset', label: 'Aset' },
    { value: 'contra_asset', label: 'Contra Aset' },
    { value: 'liability', label: 'Liabilitas' },
    { value: 'equity', label: 'Ekuitas' },
  ];

  function getAccountTypeLabel(type: string): string {
    switch (type) {
      case 'asset':
        return 'Aset';
      case 'contra_asset':
        return 'Contra Aset';
      case 'liability':
        return 'Liabilitas';
      default:
        return 'Ekuitas';
    }
  }

  $: formatCurrency = (cents: number): string => {
    return formatCurrencyHelper(cents, $currencySettings);
  };

  function resetForm() {
    name = '';
    accountType = '';
    openingBalance = '';
  }

  async function handleAddAccount() {
    if (!containerId || !name.trim() || !accountType) {
      return;
    }

    const parsed = parseFloat(openingBalance);
    if (Number.isNaN(parsed)) {
      return;
    }

    isSaving = true;
    try {
      await invoke('add_account', {
        containerId,
        name: name.trim(),
        accountType,
        openingBalance: Math.round(parsed * 100),
      });
      showAddAccount = false;
      resetForm();
      dispatch('refresh');
    } catch (error) {
      console.error('Failed to add account:', error);
      alert('Gagal menambahkan akun. Nama akun mungkin sudah ada.');
    } finally {
      isSaving = false;
    }
  }


  async function openAccount(account: typeof accounts[number]) {
    if (!containerId) return;
    selectedAccount = account;
    showDrawer = true;
    await loadAccountTransactions();
  }

  async function loadAccountTransactions() {
    if (!containerId || !selectedAccount) return;
    isLoadingTransactions = true;
    try {
      accountTransactions = await invoke<AccountTransaction[]>('get_transactions_by_account', {
        containerId,
        accountId: selectedAccount.id,
        limit: 50,
      });
    } catch (error) {
      console.error('Failed to load account transactions:', error);
      accountTransactions = [];
    } finally {
      isLoadingTransactions = false;
    }
  }

  function closeDrawer() {
    showDrawer = false;
    selectedAccount = null;
    accountTransactions = [];
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('id-ID', {
      day: '2-digit',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  }

  function isTransfer(transaction: { transfer_id: number }): boolean {
    return transaction.transfer_id !== 0;
  }

  function getTransferLabel(transaction: { amount: number; transfer_account_id: number }): string {
    const counterparty = accounts.find(acc => acc.id === transaction.transfer_account_id)?.name || 'Tanpa Akun';
    return transaction.amount >= 0 ? `Transfer dari ${counterparty}` : `Transfer ke ${counterparty}`;
  }

  $: additions = accountTransactions.reduce((sum, t) => t.amount > 0 ? sum + t.amount : sum, 0);
  $: reductions = accountTransactions.reduce((sum, t) => t.amount < 0 ? sum + Math.abs(t.amount) : sum, 0);
  $: currentBalance = selectedAccount ? selectedAccount.balance : 0;
</script>

<div class="flex h-full w-full">
  <div class="flex-1 p-4 lg:p-8 space-y-4 lg:space-y-6 overflow-auto min-w-0">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl lg:text-3xl font-black text-white mb-1">Akun</h2>
        <p class="text-xs lg:text-sm text-gray-500">Kelola akun aset, liabilitas, dan ekuitas</p>
      </div>
      <button
        on:click={() => (showAddAccount = true)}
        class="inline-flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-all"
      >
        <Plus size={16} />
        Tambah Akun
      </button>
    </div>

    <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-800 flex items-center gap-2">
        <BookOpen size={18} class="text-gray-400" />
        <h3 class="text-lg font-bold text-white">Daftar Akun</h3>
      </div>

      {#if accounts.length === 0}
        <div class="p-12 text-center">
          <div class="inline-flex p-4 bg-gray-800 rounded-full mb-4">
            <BookOpen size={32} class="text-gray-600" />
          </div>
          <p class="text-gray-400 text-lg font-medium mb-2">Belum ada akun</p>
          <p class="text-gray-600 text-sm">Klik "Tambah Akun" untuk membuat akun pertama</p>
        </div>
      {:else}
        <div class="divide-y divide-gray-800/50">
          {#each accounts as account (account.id)}
            <button type="button" on:click={() => openAccount(account)} class="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-gray-800/40 transition-colors">
              <div>
                <p class="text-white font-semibold">{account.name}</p>
                <p class="text-xs text-gray-500 uppercase tracking-wider">
                  {getAccountTypeLabel(account.account_type)}
                </p>
              </div>
              <div class="text-right">
                <p class="text-lg font-mono text-white" style="font-feature-settings: 'tnum';">
                  {formatCurrency(account.balance)}
                </p>
                <p class="text-xs text-gray-500">Saldo Saat Ini</p>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

{#if showDrawer}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50" on:click={closeDrawer}></div>
  <div class="fixed right-0 top-0 h-full w-full sm:w-[420px] bg-gray-900 border-l border-gray-800 shadow-2xl z-50 flex flex-col" in:fly={{ x: 320, duration: 200 }}>
    <div class="px-6 py-5 border-b border-gray-800 flex items-center justify-between">
      <div>
        <h3 class="text-lg font-bold text-white">{selectedAccount?.name}</h3>
        <p class="text-xs text-gray-500 uppercase tracking-wider">
          {selectedAccount ? getAccountTypeLabel(selectedAccount.account_type) : ''}
        </p>
      </div>
      <button class="p-2 hover:bg-gray-800 rounded-lg text-gray-300" on:click={closeDrawer}>
        <X size={18} />
      </button>
    </div>

    <div class="p-6 space-y-4 overflow-y-auto">
      <div class="grid grid-cols-2 gap-3">
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Saldo Awal</p>
          <p class="text-base font-mono text-white" style="font-feature-settings: 'tnum';">{selectedAccount ? formatCurrency(selectedAccount.opening_balance) : '-'}</p>
        </div>
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Saldo Saat Ini</p>
          <p class="text-base font-mono text-white" style="font-feature-settings: 'tnum';">{selectedAccount ? formatCurrency(currentBalance) : '-'}</p>
        </div>
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Penambahan</p>
          <p class="text-base font-mono text-green-400" style="font-feature-settings: 'tnum';">{formatCurrency(additions)}</p>
        </div>
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Pengurangan</p>
          <p class="text-base font-mono text-red-400" style="font-feature-settings: 'tnum';">{formatCurrency(-reductions)}</p>
        </div>
      </div>

      <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
        <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
          <h4 class="text-sm font-semibold text-gray-300">Transaksi</h4>
          <span class="text-xs text-gray-500">{accountTransactions.length} item</span>
        </div>
        {#if isLoadingTransactions}
          <div class="p-4 text-sm text-gray-400">Memuat...</div>
        {:else if accountTransactions.length === 0}
          <div class="p-4 text-sm text-gray-400">Belum ada transaksi.</div>
        {:else}
          <div class="divide-y divide-gray-800">
            {#each accountTransactions as tx (tx.id)}
              <div class="px-4 py-3 flex items-center justify-between">
                <div class="min-w-0">
                  <p class="text-sm text-white truncate">
                    {isTransfer(tx) ? (tx.description || 'Transfer') : (tx.description || tx.category || 'Transaksi')}
                  </p>
                  <p class="text-xs text-gray-500">
                    {formatDate(tx.date)} | {isTransfer(tx) ? 'Transfer' : tx.category}
                    {#if isTransfer(tx)}
                      <span class="text-blue-400"> • {getTransferLabel(tx)}</span>
                    {/if}
                  </p>
                </div>
                <p class="text-sm font-mono {tx.amount >= 0 ? 'text-green-400' : 'text-red-400'}" style="font-feature-settings: 'tnum';">
                  {tx.amount >= 0 ? '+' : ''}{formatCurrency(tx.amount)}
                </p>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

{#if showAddAccount}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" in:fade={{ duration: 200 }} out:fade={{ duration: 150 }}>
    <div class="bg-gray-900 rounded-2xl w-full max-w-md border border-gray-800 shadow-2xl overflow-hidden" in:scale={{ duration: 300, start: 0.9, easing: backOut }}>
      <div class="bg-gradient-to-r from-blue-600 to-blue-700 px-6 py-5 flex items-center justify-between rounded-t-2xl">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-white/10 rounded-lg backdrop-blur-sm">
            <BookOpen class="text-white" size={20} />
          </div>
          <div>
            <h3 class="text-xl font-bold text-white">Tambah Akun</h3>
            <p class="text-blue-100 text-xs">Buat akun baru</p>
          </div>
        </div>
        <button
          on:click={() => (showAddAccount = false)}
          class="p-2 hover:bg-white/10 rounded-lg transition-colors text-white"
        >
          <X size={20} />
        </button>
      </div>

      <form on:submit|preventDefault={handleAddAccount} class="p-6 space-y-5">
        <div>
          <label class="block text-sm font-semibold text-gray-300 mb-2">Nama Akun *</label>
          <input
            type="text"
            bind:value={name}
            placeholder="Contoh: Kas"
            class="w-full px-4 py-3 bg-gray-800 border-2 border-gray-700 rounded-xl text-white placeholder-gray-600 focus:outline-none focus:border-blue-500 transition-all"
            required
          />
        </div>

        <div>
          <label class="block text-sm font-semibold text-gray-300 mb-2">Tipe *</label>
          <Dropdown
            value={accountType}
            options={typeOptions}
            on:change={(e) => (accountType = e.detail.value)}
          />
        </div>

        <div>
          <label class="block text-sm font-semibold text-gray-300 mb-2">Saldo Awal *</label>
          <input
            type="number"
            step="0.01"
            bind:value={openingBalance}
            placeholder="0.00"
            class="w-full px-4 py-3 bg-gray-800 border-2 border-gray-700 rounded-xl text-white placeholder-gray-600 focus:outline-none focus:border-blue-500 transition-all"
            required
          />
          <p class="text-xs text-gray-500 mt-1.5">Saldo awal boleh negatif.</p>
        </div>

        <div class="flex gap-3 pt-2">
          <button
            type="submit"
            class="flex-1 bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 rounded-xl font-semibold transition-all shadow-lg shadow-blue-600/20 disabled:opacity-60"
            disabled={isSaving}
          >
            {isSaving ? 'Menyimpan...' : 'Simpan'}
          </button>
          <button
            type="button"
            on:click={() => (showAddAccount = false)}
            class="px-6 py-3 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-xl font-semibold transition-all border border-gray-700"
          >
            Batal
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
