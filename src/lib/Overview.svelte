<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, fly, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { 
    TrendingDown, 
    TrendingUp, 
    Receipt, 
    Calendar,
    UtensilsCrossed,
    ShoppingBag,
    Car,
    Sparkles,
    Receipt as ReceiptIcon,
    Home,
    Heart,
    DollarSign,
    Wallet,
    Infinity
  } from 'lucide-svelte';

  const dispatch = createEventDispatcher();

  export let monthlyBalance: number;
  export let allTimeBalance: number;
  export let selectedMonth: string;
  export let availableMonths: string[];
  export let transactions: Array<{
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
  }>;

  function formatCurrency(cents: number): string {
    const dollars = Math.abs(cents) / 100;
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(dollars);
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
    }).format(date);
  }

  function formatTime(dateString: string): string {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    }).format(date);
  }

  function groupTransactions(transactions: typeof transactions) {
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const lastWeek = new Date(today);
    lastWeek.setDate(lastWeek.getDate() - 7);

    const groups: { [key: string]: typeof transactions } = {
      Today: [],
      Yesterday: [],
      'This Week': [],
      Older: []
    };

    transactions.forEach(t => {
      const tDate = new Date(t.date);
      const tDay = new Date(tDate.getFullYear(), tDate.getMonth(), tDate.getDate());

      if (tDay.getTime() === today.getTime()) {
        groups.Today.push(t);
      } else if (tDay.getTime() === yesterday.getTime()) {
        groups.Yesterday.push(t);
      } else if (tDate >= lastWeek) {
        groups['This Week'].push(t);
      } else {
        groups.Older.push(t);
      }
    });

    return Object.entries(groups).filter(([_, txs]) => txs.length > 0);
  }

  function getCategoryColor(category: string): string {
    const colors: Record<string, string> = {
      'Food & Dining': 'bg-orange-500',
      'Shopping': 'bg-purple-500',
      'Transportation': 'bg-blue-500',
      'Entertainment': 'bg-pink-500',
      'Bills & Utilities': 'bg-yellow-500',
      'Healthcare': 'bg-red-500',
      'Income': 'bg-green-500',
      'Other': 'bg-gray-500'
    };
    return colors[category] || colors['Other'];
  }

  $: totalSpent = transactions.reduce((sum, t) => t.amount < 0 ? sum + Math.abs(t.amount) : sum, 0);
  $: totalIncome = transactions.reduce((sum, t) => t.amount > 0 ? sum + t.amount : sum, 0);
  $: transactionCount = transactions.length;
  $: dailyAverage = transactionCount > 0 ? totalSpent / 30 : 0;
  $: groupedTransactions = groupTransactions(transactions);

  function formatMonthLabel(month: string): string {
    const [year, monthNum] = month.split('-');
    const date = new Date(parseInt(year), parseInt(monthNum) - 1);
    const now = new Date();
    const currentMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
    
    if (month === currentMonth) {
      return 'This Month';
    }
    
    return date.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
  }

  function handleMonthChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    dispatch('monthChange', { month: target.value });
  }
</script>

<div class="flex h-full w-full">
  <div class="flex-1 p-8 space-y-6 overflow-auto min-w-0">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-3xl font-black text-white mb-1">Dashboard</h2>
        <p class="text-sm text-gray-500">Track your spending</p>
      </div>
      <div class="flex items-center gap-2">
        <Calendar size={18} class="text-gray-400" />
        <select
          value={selectedMonth}
          on:change={handleMonthChange}
          class="px-4 py-2.5 bg-gray-700 border-2 border-gray-600 rounded-lg text-white text-sm font-semibold focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 hover:bg-gray-600 hover:border-gray-500 transition-all cursor-pointer shadow-lg appearance-none pr-10"
          style="background-image: url('data:image/svg+xml;charset=UTF-8,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27white%27 stroke-width=%272%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpolyline points=%276 9 12 15 18 9%27%3e%3c/polyline%3e%3c/svg%3e'); background-repeat: no-repeat; background-position: right 0.5rem center; background-size: 1.5em 1.5em;"
        >
          {#each availableMonths as month}
            <option value={month} class="bg-gray-700 text-white">{formatMonthLabel(month)}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="bg-gray-950 rounded-xl p-6 border-2 border-blue-500/30 shadow-lg hover:border-blue-500/50 hover:scale-[1.02] transition-all duration-300 ease-out relative overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent pointer-events-none"></div>
        <div class="relative">
          <p class="text-gray-500 text-xs font-medium mb-3 uppercase tracking-wider">{formatMonthLabel(selectedMonth)}</p>
          <p class="text-5xl font-black text-white tracking-tight mb-1" style="font-feature-settings: 'tnum';">
            {formatCurrency(monthlyBalance)}
          </p>
          <div class="flex items-center gap-2 mt-3">
            {#if monthlyBalance >= 0}
              <TrendingUp size={14} class="text-green-400" />
              <span class="text-xs text-gray-500">Looking good</span>
            {:else}
              <TrendingDown size={14} class="text-red-400" />
              <span class="text-xs text-gray-500">In the red</span>
            {/if}
          </div>
        </div>
      </div>

      <div class="bg-gray-950 rounded-xl p-6 border-2 border-purple-500/30 shadow-lg hover:border-purple-500/50 hover:scale-[1.02] transition-all duration-300 ease-out relative overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent pointer-events-none"></div>
        <div class="relative">
          <p class="text-gray-500 text-xs font-medium mb-3 uppercase tracking-wider">All Time</p>
          <p class="text-5xl font-black text-white tracking-tight mb-1" style="font-feature-settings: 'tnum';">
            {formatCurrency(allTimeBalance)}
          </p>
          <div class="flex items-center gap-2 mt-3">
            {#if allTimeBalance >= 0}
              <TrendingUp size={14} class="text-green-400" />
              <span class="text-xs text-gray-500">Net positive</span>
            {:else}
              <TrendingDown size={14} class="text-red-400" />
              <span class="text-xs text-gray-500">Net negative</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-800">
        <h3 class="text-lg font-bold text-white">Recent Transactions</h3>
      </div>

      {#if transactions.length === 0}
        <div class="p-12 text-center">
          <div class="inline-flex p-4 bg-gray-800 rounded-full mb-4">
            <ReceiptIcon size={32} class="text-gray-600" />
          </div>
          <p class="text-gray-400 text-lg font-medium mb-2">No transactions yet</p>
          <p class="text-gray-600 text-sm">Press <kbd class="px-2 py-1 bg-gray-800 rounded text-xs">Ctrl+N</kbd> to add your first one</p>
        </div>
      {:else}
        <div>
          {#each groupedTransactions as [groupName, groupTxs]}
            <div class="border-b border-gray-800 last:border-b-0">
              <div class="px-6 py-3 bg-gray-800/50">
                <h4 class="text-xs font-bold text-gray-400 uppercase tracking-wider">{groupName}</h4>
              </div>
              <div class="divide-y divide-gray-800/50">
                {#each groupTxs as transaction, i}
                  <div class="px-6 py-4 hover:bg-gray-800/30 transition-all duration-200 group flex items-center gap-4">
                    <div class="flex-shrink-0">
                      <div class="w-2 h-2 rounded-full {getCategoryColor(transaction.category)}"></div>
                    </div>

                    <div class="flex-1 min-w-0">
                      <p class="text-white font-semibold truncate">{transaction.description || transaction.category || 'General Expense'}</p>
                      <div class="flex items-center gap-2 mt-0.5">
                        <span class="text-xs text-gray-500">{formatTime(transaction.date)}</span>
                        <span class="text-xs text-gray-700">•</span>
                        <span class="text-xs text-gray-500">{transaction.category}</span>
                      </div>
                    </div>

                    <div class="flex items-center gap-3">
                      <p class="text-lg font-mono {transaction.amount >= 0 ? 'text-green-400' : 'text-white'}" style="font-feature-settings: 'tnum';">
                        {transaction.amount >= 0 ? '+' : ''}{formatCurrency(transaction.amount)}
                      </p>
                      <button
                        on:click={() => dispatch('delete', { id: transaction.id })}
                        class="opacity-0 group-hover:opacity-100 p-2 hover:bg-red-500/10 hover:scale-110 rounded-lg text-red-400 transition-all duration-200"
                        title="Delete"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="w-96 bg-gray-900 border-l border-gray-800 flex flex-col h-full flex-shrink-0">
    <div class="p-6 pb-4 flex-shrink-0">
      <h3 class="text-lg font-bold text-white">Statistics</h3>
    </div>

    <div class="flex-1 px-6 pb-6 overflow-y-auto">
      <div class="space-y-3">
      <div class="bg-gray-800/50 rounded-xl p-5 border border-gray-700/50">
        <p class="text-gray-400 text-xs font-medium mb-4 uppercase tracking-wider">Cash Flow</p>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full bg-red-400"></div>
              <span class="text-sm text-gray-400">Spent</span>
            </div>
            <span class="text-lg font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(totalSpent)}</span>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full bg-green-400"></div>
              <span class="text-sm text-gray-400">Income</span>
            </div>
            <span class="text-lg font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(totalIncome)}</span>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">Daily Avg</p>
          <p class="text-xl font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(dailyAverage)}</p>
        </div>

        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">Count</p>
          <p class="text-xl font-mono text-white" style="font-feature-settings: 'tnum';">{transactionCount}</p>
        </div>
      </div>
      </div>
    </div>
  </div>
</div>
