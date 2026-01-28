<script lang="ts">
  export let balance: number;

  function formatCurrency(cents: number): string {
    const dollars = cents / 100;
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(dollars);
  }

  $: balanceColor = balance >= 0 ? 'text-green-400' : 'text-red-400';
</script>

<div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
  <div class="flex items-center justify-between">
    <div>
      <p class="text-gray-400 text-sm mb-1">Current Month Balance</p>
      <p class="text-4xl font-bold {balanceColor}">
        {formatCurrency(balance)}
      </p>
    </div>
    <div class="text-gray-500">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-16 w-16"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
        />
      </svg>
    </div>
  </div>
</div>
