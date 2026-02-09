<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { X, DollarSign, Plus, Trash2, ArrowLeftRight } from 'lucide-svelte';
  import Dropdown from './Dropdown.svelte';

  interface Account {
    id: number;
    name: string;
    account_type: string;
    opening_balance: number;
    container_id: number;
    created_at: string;
  }

  interface Category {
    name: string;
    category_type: 'income' | 'expense';
    is_default: boolean;
  }

  export let accounts: Account[] = [];

  const dispatch = createEventDispatcher();

  let amount = '';
  let description = '';
  let category = '';
  let transactionType: 'expense' | 'income' | 'transfer' = 'expense';
  let categories: Category[] = [];
  let showAddCategory = false;
  let newCategoryName = '';
  let accountId: number | null = null;
  let fromAccountId: number | null = null;
  let toAccountId: number | null = null;

  async function loadCategories() {
    try {
      categories = await invoke<Category[]>('get_categories');
    } catch (error) {
      console.error('Failed to load categories:', error);
      categories = [
        { name: 'Food & Dining', category_type: 'expense', is_default: true },
        { name: 'Transportation', category_type: 'expense', is_default: true },
        { name: 'Shopping', category_type: 'expense', is_default: true },
        { name: 'Entertainment', category_type: 'expense', is_default: true },
        { name: 'Bills & Utilities', category_type: 'expense', is_default: true },
        { name: 'Healthcare', category_type: 'expense', is_default: true },
        { name: 'Income', category_type: 'income', is_default: true },
        { name: 'Other', category_type: 'expense', is_default: true },
      ];
    }
  }

  async function handleAddCategory() {
    if (!newCategoryName.trim() || transactionType === 'transfer') return;

    try {
      await invoke('add_category_with_type', {
        name: newCategoryName.trim(),
        categoryType: transactionType,
      });
      await loadCategories();
      category = newCategoryName.trim();
      newCategoryName = '';
      showAddCategory = false;
    } catch (error) {
      console.error('Failed to add category:', error);
      alert('Failed to add category. It might already exist.');
    }
  }

  async function handleDeleteCategory(categoryName: string) {
    if (!confirm(`Delete category "${categoryName}"?`)) return;

    try {
      await invoke('delete_category', { name: categoryName });
      await loadCategories();
      if (category === categoryName) {
        category = '';
      }
    } catch (error) {
      console.error('Failed to delete category:', error);
      alert('Failed to delete category. Default categories cannot be deleted.');
    }
  }

  function handleSubmit() {
    const parsedAmount = parseFloat(amount);
    if (!parsedAmount) {
      return;
    }

    if (transactionType === 'transfer') {
      if (!fromAccountId || !toAccountId || fromAccountId === toAccountId) {
        return;
      }

      dispatch('transfer', {
        amount: Math.abs(parsedAmount),
        description: description.trim() || null,
        fromAccountId,
        toAccountId,
      });

      amount = '';
      description = '';
      fromAccountId = null;
      toAccountId = null;
      return;
    }

    if (accountId === null || !category) {
      return;
    }

    const selectedAccount = accounts.find(acc => acc.id === accountId);
    const isAsset = selectedAccount?.account_type === 'asset' || selectedAccount?.account_type === 'contra_asset';
    const base = Math.abs(parsedAmount);
    const signedAmount = transactionType === 'expense' ? -base : base;
    const finalAmount = isAsset ? signedAmount : -signedAmount;

    dispatch('add', {
      amount: finalAmount,
      description: description.trim() || null,
      category: category || null,
      accountId,
    });

    amount = '';
    description = '';
    category = '';
    transactionType = 'expense';
    accountId = null;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (showAddCategory) {
        showAddCategory = false;
      } else {
        dispatch('close');
      }
    }
    if (event.key === 'Enter' && event.ctrlKey && !showAddCategory) {
      handleSubmit();
    }
  }

  onMount(() => {
    loadCategories();
  });

  $: filteredCategories = transactionType === 'transfer'
    ? []
    : categories.filter(cat => cat.category_type === transactionType);

  $: categoryOptions = filteredCategories.map(cat => ({ value: cat.name, label: cat.name }));
  $: accountOptions = accounts.map(acc => ({ value: acc.id, label: acc.name }));
  $: selectedCategory = categories.find(cat => cat.name === category) || null;
  $: canDeleteCategory = selectedCategory ? !selectedCategory.is_default : false;

  $: if (transactionType !== 'transfer') {
    if (filteredCategories.length > 0 && !filteredCategories.some(cat => cat.name === category)) {
      category = filteredCategories[0].name;
    }
  }

  $: if (transactionType === 'transfer' && showAddCategory) {
    showAddCategory = false;
  }

  function handleCategoryChange(event: CustomEvent) {
    category = event.detail.value;
  }

  function handleAccountChange(event: CustomEvent) {
    accountId = event.detail.value;
  }

  function handleFromAccountChange(event: CustomEvent) {
    fromAccountId = event.detail.value;
  }

  function handleToAccountChange(event: CustomEvent) {
    toAccountId = event.detail.value;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" in:fade={{ duration: 200 }} out:fade={{ duration: 150 }}>
  <div class="bg-gray-900 rounded-2xl w-full max-w-md border border-gray-800 shadow-2xl overflow-hidden" in:scale={{ duration: 300, start: 0.9, easing: backOut }}>
    <div class="bg-gradient-to-r {transactionType === 'expense'
        ? 'from-red-600 to-red-700'
        : transactionType === 'income'
          ? 'from-green-600 to-green-700'
          : 'from-blue-600 to-blue-700'} px-6 py-5 flex items-center justify-between rounded-t-2xl">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-white/10 rounded-lg backdrop-blur-sm">
          {#if transactionType === 'transfer'}
            <ArrowLeftRight class="text-white" size={20} />
          {:else}
            <DollarSign class="text-white" size={20} />
          {/if}
        </div>
        <div>
          <h3 class="text-xl font-bold text-white">Entry Cepat</h3>
          <p class="text-xs {transactionType === 'expense'
              ? 'text-red-100'
              : transactionType === 'income'
                ? 'text-green-100'
                : 'text-blue-100'}">Masukkan Transaksi</p>
        </div>
      </div>
      <button
        on:click={() => dispatch('close')}
        class="p-2 hover:bg-white/10 rounded-lg transition-colors text-white"
      >
        <X size={20} />
      </button>
    </div>

    <form on:submit|preventDefault={handleSubmit} class="p-6 space-y-5 overflow-visible">
      <div class="overflow-visible">
        <label class="block text-sm font-semibold text-gray-300 mb-2">
          Tipe
        </label>
        <div class="flex gap-2 p-1">
          <button
            type="button"
            on:click={() => (transactionType = 'expense')}
            class="flex-1 px-4 py-3 rounded-xl font-semibold transition-all duration-300 {transactionType === 'expense'
              ? 'bg-red-500 text-white shadow-lg shadow-red-500/20 scale-105'
              : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:scale-105'}"
          >
            Expense
          </button>
          <button
            type="button"
            on:click={() => (transactionType = 'income')}
            class="flex-1 px-4 py-3 rounded-xl font-semibold transition-all duration-300 {transactionType === 'income'
              ? 'bg-green-500 text-white shadow-lg shadow-green-500/20 scale-105'
              : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:scale-105'}"
          >
            Income
          </button>
          <button
            type="button"
            on:click={() => (transactionType = 'transfer')}
            class="flex-1 px-4 py-3 rounded-xl font-semibold transition-all duration-300 {transactionType === 'transfer'
              ? 'bg-blue-500 text-white shadow-lg shadow-blue-500/20 scale-105'
              : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:scale-105'}"
          >
            Transfer
          </button>
        </div>
      </div>

      <div>
        <label for="amount" class="block text-sm font-semibold text-gray-300 mb-2">
          Jumlah *
        </label>
        <div class="relative">
          <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
            <span class="text-gray-500 text-lg">$</span>
          </div>
          <input
            id="amount"
            type="number"
            step="0.01"
            bind:value={amount}
            placeholder="0.00"
            class="w-full pl-10 pr-4 py-3 bg-gray-800 border border-gray-700 rounded-xl text-white text-lg font-semibold placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            required
          />
        </div>
      </div>

      <div>
        <label for="description" class="block text-sm font-semibold text-gray-300 mb-2">
          Deskripsi <span class="text-gray-600 font-normal">(opsional)</span>
        </label>
        <input
          id="description"
          type="text"
          bind:value={description}
          placeholder="Catatan tambahan"
          class="w-full px-4 py-3 bg-gray-800 border-2 border-gray-700 rounded-xl text-white placeholder-gray-600 focus:outline-none focus:border-blue-500 transition-all"
        />
      </div>

      {#if transactionType === 'transfer'}
        <div>
          <label class="block text-sm font-semibold text-gray-300 mb-2">
            Dari Akun *
          </label>
          <Dropdown
            value={fromAccountId}
            options={accountOptions}
            on:change={handleFromAccountChange}
            placeholder="Pilih akun asal"
          />
        </div>
        <div>
          <label class="block text-sm font-semibold text-gray-300 mb-2">
            Ke Akun *
          </label>
          <Dropdown
            value={toAccountId}
            options={accountOptions}
            on:change={handleToAccountChange}
            placeholder="Pilih akun tujuan"
          />
          {#if fromAccountId && toAccountId && fromAccountId === toAccountId}
            <p class="text-xs text-red-400 mt-1.5">Akun asal dan tujuan tidak boleh sama.</p>
          {/if}
        </div>
      {:else}
        <div>
          <label for="account" class="block text-sm font-semibold text-gray-300 mb-2">
            Akun *
          </label>
          <Dropdown
            value={accountId}
            options={accountOptions}
            on:change={handleAccountChange}
            placeholder="Pilih akun"
          />
          {#if accounts.length === 0}
            <p class="text-xs text-red-400 mt-1.5">Belum ada akun. Tambahkan di menu Akun.</p>
          {/if}
        </div>

        <div>
          <div class="flex items-center justify-between mb-2">
            <label for="category" class="block text-sm font-semibold text-gray-300">
              Category
            </label>
            <button
              type="button"
              on:click={() => (showAddCategory = !showAddCategory)}
              class="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors font-medium"
            >
              <Plus size={14} />
              Add Category
            </button>
          </div>

          {#if showAddCategory}
            <div class="mb-3 p-3 bg-gray-800 rounded-xl border border-gray-700">
              <div class="flex gap-2">
                <input
                  type="text"
                  bind:value={newCategoryName}
                  placeholder="Enter category name..."
                  class="flex-1 px-3 py-2 bg-gray-900 border-2 border-gray-600 rounded-lg text-white text-sm placeholder-gray-600 focus:outline-none focus:border-blue-500"
                  on:keydown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAddCategory())}
                />
                <button
                  type="button"
                  on:click={handleAddCategory}
                  class="px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors"
                >
                  Add
                </button>
              </div>
              <p class="text-xs text-gray-500 mt-2">Kategori baru akan disimpan sebagai {transactionType}.</p>
            </div>
          {/if}

          <Dropdown
            value={category}
            options={categoryOptions}
            on:change={handleCategoryChange}
          />

          {#if canDeleteCategory}
            <button
              type="button"
              on:click={() => handleDeleteCategory(category)}
              class="mt-2 px-3 py-1.5 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-lg transition-all text-sm flex items-center gap-1.5"
              title="Delete custom category"
            >
              <Trash2 size={14} />
              <span>Delete "{category}"</span>
            </button>
          {/if}
        </div>
      {/if}

      <div class="flex gap-3 pt-2">
        <button
          type="submit"
          class="flex-1 {transactionType === 'expense'
              ? 'bg-red-500 hover:bg-red-600 shadow-red-500/20'
              : transactionType === 'income'
                ? 'bg-green-500 hover:bg-green-600 shadow-green-500/20'
                : 'bg-blue-500 hover:bg-blue-600 shadow-blue-500/20'} text-white px-6 py-3 rounded-xl font-semibold transition-all shadow-lg disabled:opacity-60"
          disabled={transactionType === 'transfer'
            ? !fromAccountId || !toAccountId || fromAccountId === toAccountId
            : accountId === null || !category}
        >
          {transactionType === 'transfer'
            ? 'Simpan Transfer'
            : `Add ${transactionType === 'expense' ? 'Expense' : 'Income'}`}
        </button>
        <button
          type="button"
          on:click={() => dispatch('close')}
          class="px-6 py-3 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-xl font-semibold transition-all border border-gray-700"
        >
          Cancel
        </button>
      </div>

      <p class="text-xs text-gray-600 text-center pt-2">
        Press <kbd class="px-2 py-1 bg-gray-800 rounded text-gray-400">Ctrl+Enter</kbd> to submit quickly
      </p>
    </form>
  </div>
</div>
