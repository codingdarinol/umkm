<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { currencySettings, formatCurrency as formatCurrencyHelper } from './stores';

  const dispatch = createEventDispatcher();

  export let transactions: Array<{
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
  }>;

  $: formatCurrency = (cents: number): string => {
    return formatCurrencyHelper(cents, $currencySettings);
  };

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  }

  function getAmountColor(amount: number): string {
    return amount >= 0 ? 'text-green-400' : 'text-red-400';
  }
</script>

<div class="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
  {#if transactions.length === 0}
    <div class="p-8 text-center text-gray-400">
      <p>No transactions yet.</p>
      <p class="text-sm mt-2">Press Ctrl+N to add your first transaction!</p>
    </div>
  {:else}
    <div class="divide-y divide-gray-700">
      {#each transactions as transaction (transaction.id)}
        <div class="p-4 hover:bg-gray-750 transition-colors flex items-center justify-between group">
          <div class="flex-1">
            <div class="flex items-center gap-3">
              <p class="font-medium text-white">{transaction.description}</p>
              <span class="text-xs px-2 py-1 bg-gray-700 rounded text-gray-300">
                {transaction.category}
              </span>
            </div>
            <p class="text-sm text-gray-400 mt-1">
              {formatDate(transaction.date)}
            </p>
          </div>
          <div class="flex items-center gap-4">
            <p class="text-lg font-semibold {getAmountColor(transaction.amount)}">
              {formatCurrency(transaction.amount)}
            </p>
            <button
              on:click={() => dispatch('delete', { id: transaction.id })}
              class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-300 transition-all"
              title="Delete transaction"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-5 w-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                />
              </svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
