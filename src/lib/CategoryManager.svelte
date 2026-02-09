<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut, cubicOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { X, Plus, Trash2, Tag } from 'lucide-svelte';
  import Dropdown from './Dropdown.svelte';

  const dispatch = createEventDispatcher();

  interface Category {
    name: string;
    category_type: 'income' | 'expense';
    is_default: boolean;
  }

  let categories: Category[] = [];
  let newCategoryName = '';
  let newCategoryType: 'income' | 'expense' = 'expense';
  let isAdding = false;

  const typeOptions = [
    { value: 'expense', label: 'Expense' },
    { value: 'income', label: 'Income' },
  ];

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
    if (!newCategoryName.trim()) return;
    
    try {
      await invoke('add_category_with_type', { name: newCategoryName.trim(), categoryType: newCategoryType });
      await loadCategories();
      newCategoryName = '';
      newCategoryType = 'expense';
      isAdding = false;
    } catch (error) {
      console.error('Failed to add category:', error);
      alert('Failed to add category. It might already exist.');
    }
  }

  async function handleDeleteCategory(categoryName: string) {
    if (!confirm(`Delete category "${categoryName}"?\n\nThis cannot be undone and will affect existing transactions with this category.`)) return;
    
    try {
      await invoke('delete_category', { name: categoryName });
      await loadCategories();
    } catch (error) {
      console.error('Failed to delete category:', error);
      alert('Failed to delete category. Default categories cannot be deleted.');
    }
  }

  function isDefaultCategory(cat: Category): boolean {
    return cat.is_default;
  }

  onMount(() => {
    loadCategories();
  });
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" in:fade={{ duration: 200 }}>
  <div class="bg-gray-900 rounded-xl w-full max-w-2xl border border-gray-700 shadow-2xl overflow-hidden" in:scale={{ duration: 300, start: 0.95, easing: backOut }}>
    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-800">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-blue-600 rounded-lg">
          <Tag size={20} class="text-white" />
        </div>
        <div>
          <h2 class="text-xl font-bold text-white">Manage Categories</h2>
          <p class="text-sm text-gray-500">Add or remove transaction categories</p>
        </div>
      </div>
      <button
        on:click={() => dispatch('close')}
        class="p-2 hover:bg-gray-800 rounded-lg transition-colors"
      >
        <X size={20} class="text-gray-400" />
      </button>
    </div>

    <div class="p-6 max-h-[60vh] overflow-y-auto">
      <div class="mb-6">
        {#if isAdding}
          <div class="flex gap-2 p-4 bg-gray-800 rounded-xl border border-gray-700">
            <input
              type="text"
              bind:value={newCategoryName}
              placeholder="Enter category name..."
              class="flex-1 px-4 py-2 bg-gray-900 border-2 border-gray-600 rounded-lg text-white placeholder-gray-600 focus:outline-none focus:border-blue-500"
              on:keydown={(e) => e.key === 'Enter' && handleAddCategory()}
              autofocus
            />
            <div class="min-w-[120px]">
              <Dropdown
                value={newCategoryType}
                options={typeOptions}
                on:change={(e) => (newCategoryType = e.detail.value)}
              />
            </div>
            <button
              type="button"
              on:click={handleAddCategory}
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors"
            >
              Add
            </button>
            <button
              type="button"
              on:click={() => (isAdding = false, newCategoryName = '')}
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium transition-colors"
            >
              Cancel
            </button>
          </div>
        {:else}
          <button
            type="button"
            on:click={() => (isAdding = true)}
            class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-xl font-medium transition-colors"
          >
            <Plus size={18} />
            Add New Category
          </button>
        {/if}
      </div>

      <div class="space-y-2">
        <h3 class="text-sm font-semibold text-gray-400 mb-3 uppercase tracking-wider">All Categories ({categories.length})</h3>
        {#each categories as cat}
          <div class="flex items-center justify-between p-4 bg-gray-800 rounded-xl hover:bg-gray-750 hover:scale-[1.01] transition-all duration-200 border border-gray-700">
            <div class="flex items-center gap-3">
              <div class="w-3 h-3 rounded-full {isDefaultCategory(cat) ? 'bg-green-500' : 'bg-blue-500'}"></div>
              <span class="text-white font-medium">{cat.name}</span>
              <span class="text-xs px-2 py-0.5 rounded-full {cat.category_type === 'income' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'}">
                {cat.category_type === 'income' ? 'Income' : 'Expense'}
              </span>
              {#if isDefaultCategory(cat)}
                <span class="text-xs px-2 py-0.5 bg-green-500/20 text-green-400 rounded-full">Default</span>
              {/if}
            </div>
            {#if !isDefaultCategory(cat)}
              <button
                type="button"
                on:click={() => handleDeleteCategory(cat.name)}
                class="p-2 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-lg transition-all"
                title="Delete category"
              >
                <Trash2 size={16} />
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <div class="px-6 py-4 border-t border-gray-800 bg-gray-900/50">
      <p class="text-xs text-gray-500">
        Note: Default categories cannot be deleted. Custom categories can be removed but will remain in existing transactions.
      </p>
    </div>
  </div>
</div>
