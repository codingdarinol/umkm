<script lang="ts">
  import { fly } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';
  import { Tag, X, Plus, Trash2, Pencil } from 'lucide-svelte';
  import Dropdown from './Dropdown.svelte';
  import { currencySettings, formatCurrency as formatCurrencyHelper } from './stores';

  export let containerId: number | null = null;

  interface Category {
    name: string;
    category_type: 'income' | 'expense';
    is_default: boolean;
  }

  interface Account {
    id: number;
    name: string;
  }

  interface CategoryTransaction {
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
    account_id: number;
    transfer_id: number;
    transfer_account_id: number;
  }

  const DEFAULT_LIMIT = 50;
  const typeOptions = [
    { value: 'expense', label: 'Expense' },
    { value: 'income', label: 'Income' },
  ];

  let categories: Category[] = [];
  let accounts: Account[] = [];
  let selectedCategory: Category | null = null;
  let categoryTransactions: CategoryTransaction[] = [];
  let isLoadingCategories = false;
  let isLoadingTransactions = false;
  let isAddingCategory = false;
  let isUpdatingCategory = false;
  let deletingCategoryName = '';
  let showDrawer = false;
  let showAddCategory = false;
  let showEditCategory = false;
  let lastContainerId: number | null = null;
  let newCategoryName = '';
  let newCategoryType: Category['category_type'] = 'expense';
  let editCategoryName = '';
  let editCategoryType: Category['category_type'] = 'expense';

  $: formatCurrency = (cents: number): string => {
    return formatCurrencyHelper(cents, $currencySettings);
  };

  function getCategoryTypeLabel(type: Category['category_type']): string {
    return type === 'income' ? 'Income' : 'Expense';
  }

  function getAccountName(accountId: number): string {
    if (!accountId) return 'Tanpa Akun';
    return accounts.find(acc => acc.id === accountId)?.name || 'Tanpa Akun';
  }

  function isTransfer(transaction: { transfer_id: number }): boolean {
    return transaction.transfer_id !== 0;
  }

  function getTransferLabel(transaction: { amount: number; transfer_account_id: number }): string {
    const counterparty = getAccountName(transaction.transfer_account_id);
    return transaction.amount >= 0 ? `Transfer dari ${counterparty}` : `Transfer ke ${counterparty}`;
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

  async function loadCategories() {
    isLoadingCategories = true;
    try {
      categories = await invoke<Category[]>('get_categories');
    } catch (error) {
      console.error('Failed to load categories:', error);
      categories = [];
    } finally {
      isLoadingCategories = false;
    }
  }

  async function loadAccounts() {
    if (!containerId) {
      accounts = [];
      return;
    }

    try {
      accounts = await invoke<Account[]>('get_accounts', { containerId });
    } catch (error) {
      console.error('Failed to load accounts for ledger:', error);
      accounts = [];
    }
  }

  async function handleAddCategory() {
    if (!newCategoryName.trim()) return;

    isAddingCategory = true;
    try {
      await invoke('add_category_with_type', {
        name: newCategoryName.trim(),
        categoryType: newCategoryType,
      });
      await loadCategories();
      newCategoryName = '';
      newCategoryType = 'expense';
      showAddCategory = false;
    } catch (error) {
      console.error('Failed to add category from ledger:', error);
      alert('Gagal menambahkan kategori. Nama mungkin sudah ada.');
    } finally {
      isAddingCategory = false;
    }
  }

  async function handleDeleteCategory(category: Category) {
    if (category.is_default) return;
    if (!confirm(`Hapus kategori "${category.name}"?\n\nTransaksi lama tetap tersimpan dengan nama kategori ini.`)) {
      return;
    }

    deletingCategoryName = category.name;
    try {
      await invoke('delete_category', { name: category.name });
      await loadCategories();
      if (selectedCategory?.name === category.name) {
        closeDrawer();
      }
    } catch (error) {
      console.error('Failed to delete category from ledger:', error);
      alert('Gagal menghapus kategori.');
    } finally {
      deletingCategoryName = '';
    }
  }

  function startEditCategory() {
    if (!selectedCategory) return;
    editCategoryName = selectedCategory.name;
    editCategoryType = selectedCategory.category_type;
    showEditCategory = true;
  }

  function resetEditCategoryForm() {
    editCategoryName = '';
    editCategoryType = 'expense';
  }

  async function handleUpdateCategory() {
    if (!selectedCategory) return;

    const newName = editCategoryName.trim();
    if (!newName) return;

    const oldName = selectedCategory.name;
    const isDefault = selectedCategory.is_default;
    isUpdatingCategory = true;
    try {
      await invoke('update_category', {
        oldName,
        newName,
        categoryType: editCategoryType,
      });

      selectedCategory = {
        name: newName,
        category_type: editCategoryType,
        is_default: isDefault,
      };
      showEditCategory = false;
      resetEditCategoryForm();
      await loadCategories();
      await loadCategoryTransactions();
    } catch (error) {
      console.error('Failed to update category from ledger:', error);
      alert('Gagal mengubah kategori. Nama mungkin sudah ada.');
    } finally {
      isUpdatingCategory = false;
    }
  }

  async function openCategory(category: Category) {
    selectedCategory = category;
    showDrawer = true;
    await loadCategoryTransactions();
  }

  async function loadCategoryTransactions() {
    if (!containerId || !selectedCategory) return;

    isLoadingTransactions = true;
    try {
      categoryTransactions = await invoke<CategoryTransaction[]>('get_transactions_by_category', {
        containerId,
        category: selectedCategory.name,
        limit: DEFAULT_LIMIT,
      });
    } catch (error) {
      console.error('Failed to load category transactions:', error);
      categoryTransactions = [];
    } finally {
      isLoadingTransactions = false;
    }
  }

  function closeDrawer() {
    showDrawer = false;
    selectedCategory = null;
    categoryTransactions = [];
    showEditCategory = false;
    resetEditCategoryForm();
  }

  $: if (containerId !== lastContainerId) {
    lastContainerId = containerId;
    closeDrawer();

    if (containerId) {
      loadCategories();
      loadAccounts();
      showAddCategory = false;
      showEditCategory = false;
      newCategoryName = '';
      newCategoryType = 'expense';
      resetEditCategoryForm();
    } else {
      categories = [];
      accounts = [];
    }
  }

  $: if (selectedCategory) {
    const match = categories.find(cat => cat.name === selectedCategory?.name);
    if (!match && showDrawer) {
      closeDrawer();
    }
  }

  $: additions = categoryTransactions.reduce((sum, t) => t.amount > 0 ? sum + t.amount : sum, 0);
  $: reductions = categoryTransactions.reduce((sum, t) => t.amount < 0 ? sum + Math.abs(t.amount) : sum, 0);
  $: totalFlow = additions + reductions;
</script>

<div class="flex h-full w-full">
  <div class="flex-1 p-4 lg:p-8 space-y-4 lg:space-y-6 overflow-auto min-w-0">
    <div class="flex items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl lg:text-3xl font-black text-white mb-1">Ledger</h2>
        <p class="text-xs lg:text-sm text-gray-500">Daftar kategori transaksi</p>
      </div>
      <button
        type="button"
        on:click={() => (showAddCategory = !showAddCategory)}
        class="inline-flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-all"
      >
        <Plus size={16} />
        Tambah Kategori
      </button>
    </div>

    {#if showAddCategory}
      <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg p-4">
        <div class="flex flex-col lg:flex-row gap-3">
          <input
            type="text"
            bind:value={newCategoryName}
            placeholder="Nama kategori baru"
            class="flex-1 px-4 py-2.5 bg-gray-800 border-2 border-gray-700 rounded-lg text-white placeholder-gray-600 focus:outline-none focus:border-blue-500 transition-all"
            on:keydown={(e) => e.key === 'Enter' && handleAddCategory()}
          />
          <div class="lg:w-44">
            <Dropdown
              value={newCategoryType}
              options={typeOptions}
              on:change={(e) => (newCategoryType = e.detail.value)}
            />
          </div>
          <div class="flex gap-2">
            <button
              type="button"
              on:click={handleAddCategory}
              disabled={isAddingCategory}
              class="px-4 py-2.5 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-all disabled:opacity-60"
            >
              {isAddingCategory ? 'Menyimpan...' : 'Simpan'}
            </button>
            <button
              type="button"
              on:click={() => {
                showAddCategory = false;
                newCategoryName = '';
                newCategoryType = 'expense';
              }}
              class="px-4 py-2.5 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg font-medium transition-all border border-gray-700"
            >
              Batal
            </button>
          </div>
        </div>
      </div>
    {/if}

    <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-800 flex items-center gap-2">
        <Tag size={18} class="text-gray-400" />
        <h3 class="text-lg font-bold text-white">Daftar Kategori</h3>
      </div>

      {#if isLoadingCategories}
        <div class="p-6 text-sm text-gray-400">Memuat kategori...</div>
      {:else if categories.length === 0}
        <div class="p-12 text-center">
          <div class="inline-flex p-4 bg-gray-800 rounded-full mb-4">
            <Tag size={32} class="text-gray-600" />
          </div>
          <p class="text-gray-400 text-lg font-medium mb-2">Belum ada kategori</p>
          <p class="text-gray-600 text-sm">Tambahkan kategori dari menu Commands jika dibutuhkan.</p>
        </div>
      {:else}
        <div class="divide-y divide-gray-800/50">
          {#each categories as category (category.name)}
            <button
              type="button"
              on:click={() => openCategory(category)}
              class="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-gray-800/40 transition-colors"
            >
              <div class="flex items-center gap-3 min-w-0">
                <span class="w-2.5 h-2.5 rounded-full {category.category_type === 'income' ? 'bg-green-400' : 'bg-red-400'}"></span>
                <div class="min-w-0">
                  <p class="text-white font-semibold truncate">{category.name}</p>
                  <p class="text-xs text-gray-500 uppercase tracking-wider">{getCategoryTypeLabel(category.category_type)}</p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs px-2 py-1 rounded-full {category.is_default ? 'bg-green-500/20 text-green-300' : 'bg-blue-500/20 text-blue-300'}">
                  {category.is_default ? 'Default' : 'Custom'}
                </span>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

{#if showDrawer && selectedCategory}
  <button
    type="button"
    class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50"
    on:click={closeDrawer}
    aria-label="Tutup drawer ledger"
  ></button>
  <div class="fixed right-0 top-0 h-full w-full sm:w-[420px] bg-gray-900 border-l border-gray-800 shadow-2xl z-50 flex flex-col" in:fly={{ x: 320, duration: 200 }}>
    <div class="px-6 py-5 border-b border-gray-800 flex items-center justify-between">
      <div>
        <h3 class="text-lg font-bold text-white">{selectedCategory.name}</h3>
        <p class="text-xs text-gray-500 uppercase tracking-wider">{getCategoryTypeLabel(selectedCategory.category_type)}</p>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="p-2 hover:bg-gray-800 rounded-lg text-gray-300"
          title="Edit kategori"
          on:click={startEditCategory}
        >
          <Pencil size={16} />
        </button>
        {#if !selectedCategory.is_default}
          <button
            type="button"
            class="p-2 hover:bg-red-500/10 rounded-lg text-red-400 disabled:opacity-60"
            title="Hapus kategori"
            on:click={() => handleDeleteCategory(selectedCategory)}
            disabled={deletingCategoryName === selectedCategory.name}
          >
            <Trash2 size={16} />
          </button>
        {/if}
        <button class="p-2 hover:bg-gray-800 rounded-lg text-gray-300" on:click={closeDrawer}>
          <X size={18} />
        </button>
      </div>
    </div>

    <div class="p-6 space-y-4 overflow-y-auto">
      <div class="grid grid-cols-2 gap-3">
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Total Arus</p>
          <p class="text-base font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(totalFlow)}</p>
        </div>
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Jumlah Transaksi</p>
          <p class="text-base font-mono text-white" style="font-feature-settings: 'tnum';">{categoryTransactions.length}</p>
        </div>
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Masuk</p>
          <p class="text-base font-mono text-green-400" style="font-feature-settings: 'tnum';">{formatCurrency(additions)}</p>
        </div>
        <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
          <p class="text-xs text-gray-500 mb-1">Keluar</p>
          <p class="text-base font-mono text-red-400" style="font-feature-settings: 'tnum';">{formatCurrency(-reductions)}</p>
        </div>
      </div>

      <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
        <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
          <h4 class="text-sm font-semibold text-gray-300">Transaksi</h4>
          <span class="text-xs text-gray-500">Maks {DEFAULT_LIMIT} item</span>
        </div>

        {#if isLoadingTransactions}
          <div class="p-4 text-sm text-gray-400">Memuat...</div>
        {:else if categoryTransactions.length === 0}
          <div class="p-4 text-sm text-gray-400">Belum ada transaksi untuk kategori ini.</div>
        {:else}
          <div class="divide-y divide-gray-800">
            {#each categoryTransactions as tx (tx.id)}
              <div class="px-4 py-3 flex items-center justify-between">
                <div class="min-w-0">
                  <p class="text-sm text-white truncate">{tx.description || tx.category || 'Transaksi'}</p>
                  <p class="text-xs text-gray-500">
                    {formatDate(tx.date)} | {getAccountName(tx.account_id)}
                    {#if isTransfer(tx)}
                      <span class="text-blue-400"> | {getTransferLabel(tx)}</span>
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

{#if showEditCategory && selectedCategory}
  <button
    type="button"
    class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[60]"
    on:click={() => {
      showEditCategory = false;
      resetEditCategoryForm();
    }}
    aria-label="Tutup form edit kategori"
  ></button>
  <div class="fixed inset-0 z-[61] flex items-center justify-center p-4">
    <div class="bg-gray-900 rounded-2xl w-full max-w-md border border-gray-800 shadow-2xl overflow-hidden">
      <div class="bg-gradient-to-r from-purple-600 to-purple-700 px-6 py-5 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-white/10 rounded-lg backdrop-blur-sm">
            <Pencil class="text-white" size={20} />
          </div>
          <div>
            <h3 class="text-xl font-bold text-white">Edit Kategori</h3>
            <p class="text-purple-100 text-xs">Perbarui nama dan tipe kategori</p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => {
            showEditCategory = false;
            resetEditCategoryForm();
          }}
          class="p-2 hover:bg-white/10 rounded-lg transition-colors text-white"
        >
          <X size={20} />
        </button>
      </div>

      <form on:submit|preventDefault={handleUpdateCategory} class="p-6 space-y-5">
        <div>
          <label for="ledger-edit-category-name" class="block text-sm font-semibold text-gray-300 mb-2">Nama Kategori *</label>
          <input
            id="ledger-edit-category-name"
            type="text"
            bind:value={editCategoryName}
            class="w-full px-4 py-3 bg-gray-800 border-2 border-gray-700 rounded-xl text-white placeholder-gray-600 focus:outline-none focus:border-purple-500 transition-all"
            required
          />
        </div>

        <div>
          <p class="block text-sm font-semibold text-gray-300 mb-2">Tipe *</p>
          <Dropdown
            value={editCategoryType}
            options={typeOptions}
            on:change={(e) => (editCategoryType = e.detail.value)}
          />
        </div>

        <div class="flex gap-3 pt-2">
          <button
            type="submit"
            class="flex-1 bg-purple-600 hover:bg-purple-700 text-white px-6 py-3 rounded-xl font-semibold transition-all shadow-lg shadow-purple-600/20 disabled:opacity-60"
            disabled={isUpdatingCategory}
          >
            {isUpdatingCategory ? 'Menyimpan...' : 'Simpan'}
          </button>
          <button
            type="button"
            on:click={() => {
              showEditCategory = false;
              resetEditCategoryForm();
            }}
            class="px-6 py-3 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-xl font-semibold transition-all border border-gray-700"
          >
            Batal
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
