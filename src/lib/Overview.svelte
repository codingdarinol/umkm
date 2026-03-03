<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { fade, fly, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { currencySettings, formatCurrency as formatCurrencyHelper } from './stores';
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
  import Dropdown from './Dropdown.svelte';

  const dispatch = createEventDispatcher();

  interface ProfitLossReport {
    total_income: number;
    total_expense: number;
  }

  interface BalanceSheetReport {
    total_assets: number;
    total_liabilities: number;
  }

  export let monthlyBalance: number;
  export let allTimeBalance: number;
  export let containerId: number | null = null;
  export let selectedMonth: string;
  export let availableMonths: string[];
  export let transactions: Array<{
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
    account_id: number;
    transfer_id: number;
    transfer_account_id: number;
  }>;

  export let accounts: Array<{
    id: number;
    name: string;
    account_type: string;
  }> = [];

  let isLoadingStats = false;
  let currentProfitLoss: ProfitLossReport | null = null;
  let previousProfitLoss: ProfitLossReport | null = null;
  let topExpenseCategory: [string, number] | null = null;
  let netWorthSeries: Array<{ month: string; value: number }> = [];
  let lastStatsKey = '';
  let selectedYearValue = '';
  let selectedMonthValue = '';
  let selectedDate = '';
  let dateFilterInput: HTMLInputElement | null = null;
  let lastSyncedMonth = '';
  const MONTH_NAMES_ID = [
    'Januari',
    'Februari',
    'Maret',
    'April',
    'Mei',
    'Juni',
    'Juli',
    'Agustus',
    'September',
    'Oktober',
    'November',
    'Desember',
  ];

  $: formatCurrency = (cents: number): string => {
    return formatCurrencyHelper(cents, $currencySettings);
  };

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

  function getDisplayCategory(transaction: { category: string; transfer_id: number }): string {
    return isTransfer(transaction) ? 'Transfer' : transaction.category;
  }

  function formatTime(dateString: string): string {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('id-ID', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    }).format(date);
  }

  function groupTransactionsByDate(txList: Array<{
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
  }>) {
    const grouped = new Map<string, typeof txList>();

    txList.forEach((tx) => {
      const dateKey = tx.date.slice(0, 10);
      if (!grouped.has(dateKey)) {
        grouped.set(dateKey, []);
      }
      grouped.get(dateKey)?.push(tx);
    });

    return Array.from(grouped.entries())
      .sort(([a], [b]) => b.localeCompare(a))
      .map(([dateKey, txs]) => {
        const parsed = new Date(`${dateKey}T00:00:00`);
        const label = parsed.toLocaleDateString('id-ID', {
          weekday: 'long',
          day: '2-digit',
          month: 'long',
          year: 'numeric',
        });
        return [label, txs];
      });
  }

  function getCategoryColor(category: string): string {
    const colors: Record<string, string> = {
      'Biaya Gaji': 'bg-orange-500',
      'Beban Penyusutan dan Amortisasi': 'bg-purple-500',
      'Beban Transportasi': 'bg-blue-500',
      'Beban Sewa': 'bg-pink-500',
      'Beban Umum dan Administrasi': 'bg-yellow-500',
      'Beban Pemasaran atau Promosi': 'bg-red-500',
      'Penjualan': 'bg-green-500',
      'Transfer': 'bg-blue-400',
      'Beban Usaha Lainnya': 'bg-gray-500'
    };
    return colors[category] || colors['Beban Usaha Lainnya'];
  }

  $: displayedTransactions = selectedDate
    ? transactions.filter((t) => t.date.slice(0, 10) === selectedDate)
    : transactions;

  $: totalSpent = displayedTransactions.reduce((sum, t) => {
    if (isTransfer(t)) return sum;
    return t.amount < 0 ? sum + Math.abs(t.amount) : sum;
  }, 0);
  $: totalIncome = displayedTransactions.reduce((sum, t) => {
    if (isTransfer(t)) return sum;
    return t.amount > 0 ? sum + t.amount : sum;
  }, 0);
  $: transactionCount = displayedTransactions.length;
  $: daysInMonth = getDaysInMonth(selectedMonth);
  $: useDateFilterActive = Boolean(selectedDate);
  $: totalExpenseForStats = useDateFilterActive ? totalSpent : (currentProfitLoss?.total_expense ?? totalSpent);
  $: totalIncomeForStats = useDateFilterActive ? totalIncome : (currentProfitLoss?.total_income ?? totalIncome);
  $: dailyAverage = useDateFilterActive ? totalExpenseForStats : (daysInMonth > 0 ? totalExpenseForStats / daysInMonth : 0);
  $: savingRate = totalIncomeForStats > 0
    ? ((totalIncomeForStats - totalExpenseForStats) / totalIncomeForStats) * 100
    : 0;
  $: expenseMoMChange = useDateFilterActive
    ? null
    : calculatePercentChange(
      previousProfitLoss?.total_expense ?? 0,
      currentProfitLoss?.total_expense ?? totalSpent,
    );
  $: incomeMoMChange = useDateFilterActive
    ? null
    : calculatePercentChange(
      previousProfitLoss?.total_income ?? 0,
      currentProfitLoss?.total_income ?? totalIncome,
    );
  $: netWorthCurrent = netWorthSeries.length > 0 ? netWorthSeries[netWorthSeries.length - 1].value : 0;
  $: netWorthPrevious = netWorthSeries.length > 1 ? netWorthSeries[netWorthSeries.length - 2].value : 0;
  $: netWorthChange = calculatePercentChange(netWorthPrevious, netWorthCurrent);
  $: netWorthPoints = buildSparklinePoints(netWorthSeries.map(item => item.value), 260, 88, 8);
  $: topExpenseCategoryForView = useDateFilterActive
    ? getTopExpenseCategory(displayedTransactions)
    : topExpenseCategory;
  $: groupedTransactions = groupTransactionsByDate(displayedTransactions);

  function formatMonthLabel(month: string): string {
    const [year, monthNum] = month.split('-');
    const date = new Date(parseInt(year), parseInt(monthNum) - 1);
    const now = new Date();
    const currentMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
    
    if (month === currentMonth) {
      return 'Bulan Ini';
    }
    
    return date.toLocaleDateString('id-ID', { month: 'long', year: 'numeric' });
  }

  function buildYearOptions(months: string[]): string[] {
    const years = new Set<string>();
    const currentYear = String(new Date().getFullYear());
    years.add(currentYear);

    months.forEach((month) => {
      const [year] = month.split('-');
      if (year) years.add(year);
    });

    return Array.from(years).sort((a, b) => b.localeCompare(a));
  }

  $: if (selectedMonth && selectedMonth !== lastSyncedMonth) {
    const [yearPart, monthPart] = selectedMonth.split('-');
    if (yearPart && monthPart) {
      selectedYearValue = yearPart;
      selectedMonthValue = monthPart;
      lastSyncedMonth = selectedMonth;
    }
  }

  $: yearOptions = buildYearOptions(availableMonths).map((year) => ({
    value: year,
    label: year,
  }));

  $: monthOptions = MONTH_NAMES_ID.map((name, index) => {
    const value = String(index + 1).padStart(2, '0');
    return { value, label: name };
  });

  function dispatchMonthSelection() {
    if (!selectedYearValue || !selectedMonthValue) return;
    dispatch('monthChange', { month: `${selectedYearValue}-${selectedMonthValue}` });
  }

  function handleYearChange(event: CustomEvent) {
    selectedYearValue = String(event.detail.value);
    selectedDate = '';
    dispatchMonthSelection();
  }

  function handleMonthPartChange(event: CustomEvent) {
    selectedMonthValue = String(event.detail.value);
    selectedDate = '';
    dispatchMonthSelection();
  }

  function openDatePicker() {
    if (!dateFilterInput) return;
    if (typeof dateFilterInput.showPicker === 'function') {
      dateFilterInput.showPicker();
      return;
    }
    dateFilterInput.focus();
    dateFilterInput.click();
  }

  function handleDateFilterChange(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    const value = target.value;
    selectedDate = value;
    if (!value) return;

    const monthFromDate = value.slice(0, 7);
    if (monthFromDate !== selectedMonth) {
      dispatch('monthChange', { month: monthFromDate });
    }
  }

  function clearDateFilter() {
    selectedDate = '';
  }

  function formatSelectedDate(dateValue: string): string {
    if (!dateValue) return '';
    const parsed = new Date(`${dateValue}T00:00:00`);
    return parsed.toLocaleDateString('id-ID', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
    });
  }

  function parseMonth(month: string): [number, number] {
    const [yearStr, monthStr] = month.split('-');
    return [parseInt(yearStr), parseInt(monthStr)];
  }

  function getDaysInMonth(month: string): number {
    if (!month) return 30;
    const [year, monthNum] = parseMonth(month);
    return new Date(year, monthNum, 0).getDate();
  }

  function getPreviousMonth(month: string): string {
    const [year, monthNum] = parseMonth(month);
    const date = new Date(year, monthNum - 1, 1);
    date.setMonth(date.getMonth() - 1);
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
  }

  function getRecentMonths(endMonth: string, count: number): string[] {
    const [year, monthNum] = parseMonth(endMonth);
    const endDate = new Date(year, monthNum - 1, 1);
    const months: string[] = [];

    for (let i = count - 1; i >= 0; i--) {
      const d = new Date(endDate);
      d.setMonth(d.getMonth() - i);
      months.push(`${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`);
    }
    return months;
  }

  function calculatePercentChange(previous: number, current: number): number | null {
    if (previous === 0) return null;
    return ((current - previous) / previous) * 100;
  }

  function formatPercent(value: number | null): string {
    if (value === null || Number.isNaN(value)) return '-';
    const sign = value > 0 ? '+' : '';
    return `${sign}${value.toFixed(1)}%`;
  }

  function getTopExpenseCategory(txList: Array<{ category: string; amount: number; transfer_id: number }>): [string, number] | null {
    const totals = new Map<string, number>();
    txList.forEach((tx) => {
      if (isTransfer(tx) || tx.amount >= 0) return;
      const current = totals.get(tx.category) ?? 0;
      totals.set(tx.category, current + Math.abs(tx.amount));
    });

    let top: [string, number] | null = null;
    totals.forEach((value, key) => {
      if (!top || value > top[1]) {
        top = [key, value];
      }
    });
    return top;
  }

  function buildSparklinePoints(values: number[], width: number, height: number, padding: number): string {
    if (values.length === 0) return '';
    if (values.length === 1) {
      const y = height / 2;
      return `${padding},${y} ${width - padding},${y}`;
    }

    const min = Math.min(...values);
    const max = Math.max(...values);
    const spread = max - min || 1;
    const innerWidth = width - padding * 2;
    const innerHeight = height - padding * 2;

    return values
      .map((value, index) => {
        const x = padding + (index / (values.length - 1)) * innerWidth;
        const y = padding + (max - value) / spread * innerHeight;
        return `${x},${y}`;
      })
      .join(' ');
  }

  async function loadStatistics() {
    if (!containerId || !selectedMonth) return;

    isLoadingStats = true;
    try {
      const previousMonth = getPreviousMonth(selectedMonth);
      const monthsForNetWorth = getRecentMonths(selectedMonth, 6);

      const [currentPL, previousPL, categoryTotalsForMonth, balanceSheets] = await Promise.all([
        invoke<ProfitLossReport>('get_profit_and_loss_for_month', { containerId, month: selectedMonth }),
        invoke<ProfitLossReport>('get_profit_and_loss_for_month', { containerId, month: previousMonth }),
        invoke<Array<[string, number]>>('get_category_totals_for_month', { containerId, month: selectedMonth }),
        Promise.all(
          monthsForNetWorth.map((month) =>
            invoke<BalanceSheetReport>('get_balance_sheet_for_month', { containerId, month })
          )
        ),
      ]);

      currentProfitLoss = currentPL;
      previousProfitLoss = previousPL;
      topExpenseCategory = categoryTotalsForMonth.length > 0 ? categoryTotalsForMonth[0] : null;
      netWorthSeries = monthsForNetWorth.map((month, index) => ({
        month,
        value: balanceSheets[index].total_assets - balanceSheets[index].total_liabilities,
      }));
    } catch (error) {
      console.error('Failed to load overview statistics:', error);
      currentProfitLoss = null;
      previousProfitLoss = null;
      topExpenseCategory = null;
      netWorthSeries = [];
    } finally {
      isLoadingStats = false;
    }
  }

  $: statsKey = `${containerId ?? 'none'}-${selectedMonth ?? ''}`;
  $: if (containerId && selectedMonth && statsKey !== lastStatsKey) {
    lastStatsKey = statsKey;
    loadStatistics();
  }
</script>

<div class="flex h-full w-full">
  <div class="flex-1 p-4 lg:p-8 space-y-4 lg:space-y-6 overflow-auto min-w-0">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-0">
      <div>
        <h2 class="text-2xl lg:text-3xl font-black text-white mb-1">Dasbor</h2>
        <p class="text-xs lg:text-sm text-gray-500">Pantau Keuangan Anda</p>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <input
          bind:this={dateFilterInput}
          type="date"
          bind:value={selectedDate}
          on:change={handleDateFilterChange}
          class="absolute opacity-0 pointer-events-none w-0 h-0"
          tabindex="-1"
          aria-hidden="true"
        />
        <button
          type="button"
          on:click={openDatePicker}
          class="h-10 w-10 flex items-center justify-center rounded-lg border {selectedDate ? 'border-blue-500 bg-blue-500/10 text-blue-300' : 'border-gray-700 bg-gray-900 text-gray-400 hover:text-white'} transition-colors"
          title="Filter tanggal"
        >
          <Calendar size={18} />
        </button>
        {#if selectedDate}
          <button
            type="button"
            on:click={clearDateFilter}
            class="h-10 px-3 bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg text-xs text-gray-300 transition-colors"
          >
            Reset
          </button>
        {/if}
        <div class="w-[110px]">
          <Dropdown
            value={selectedYearValue}
            options={yearOptions}
            on:change={handleYearChange}
          />
        </div>
        <div class="w-[150px]">
          <Dropdown
            value={selectedMonthValue}
            options={monthOptions}
            on:change={handleMonthPartChange}
          />
        </div>
      </div>
    </div>

    {#if selectedDate}
      <div class="px-3 py-2 bg-blue-500/10 border border-blue-500/30 rounded-lg">
        <p class="text-xs text-blue-200">Filter tanggal aktif: {formatSelectedDate(selectedDate)}</p>
      </div>
    {/if}

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 lg:gap-4">
      <div class="bg-gray-950 rounded-xl p-6 border-2 {monthlyBalance >= 0 ? 'border-blue-500/30 hover:border-blue-500/50' : 'border-red-500/30 hover:border-red-500/50'} shadow-lg hover:scale-[1.02] transition-all duration-300 ease-out relative overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-br {monthlyBalance >= 0 ? 'from-blue-500/5' : 'from-red-500/5'} to-transparent pointer-events-none"></div>
        <div class="relative">
          <p class="text-gray-500 text-xs font-medium mb-2 lg:mb-3 uppercase tracking-wider">{formatMonthLabel(selectedMonth)}</p>
          <p class="text-2xl lg:text-3xl xl:text-4xl font-black tracking-tight mb-1 {monthlyBalance >= 0 ? 'text-white' : 'text-red-400'}" style="font-feature-settings: 'tnum';">
            {formatCurrency(monthlyBalance)}
          </p>
          <div class="flex items-center gap-2 mt-3">
            {#if monthlyBalance === 0}
              <span class="text-xs text-gray-500">Impas</span>
            {:else if monthlyBalance > 0}
              <TrendingUp size={14} class="text-green-400" />
              <span class="text-xs text-gray-500">Positif</span>
            {:else}
              <TrendingDown size={14} class="text-red-400" />
              <span class="text-xs text-gray-500">Negatif</span>
            {/if}
          </div>
        </div>
      </div>

      <div class="bg-gray-950 rounded-xl p-6 border-2 {allTimeBalance >= 0 ? 'border-purple-500/30 hover:border-purple-500/50' : 'border-red-500/30 hover:border-red-500/50'} shadow-lg hover:scale-[1.02] transition-all duration-300 ease-out relative overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-br {allTimeBalance >= 0 ? 'from-purple-500/5' : 'from-red-500/5'} to-transparent pointer-events-none"></div>
        <div class="relative">
          <p class="text-gray-500 text-xs font-medium mb-2 lg:mb-3 uppercase tracking-wider">Sepanjang Waktu</p>
          <p class="text-2xl lg:text-3xl xl:text-4xl font-black tracking-tight mb-1 {allTimeBalance >= 0 ? 'text-white' : 'text-red-400'}" style="font-feature-settings: 'tnum';">
            {formatCurrency(allTimeBalance)}
          </p>
          <div class="flex items-center gap-2 mt-3">
            {#if allTimeBalance === 0}
              <span class="text-xs text-gray-500">Impas</span>
            {:else if allTimeBalance > 0}
              <TrendingUp size={14} class="text-green-400" />
              <span class="text-xs text-gray-500">Net Positif</span>
            {:else}
              <TrendingDown size={14} class="text-red-400" />
              <span class="text-xs text-gray-500">Net Negatif</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-800">
        <h3 class="text-lg font-bold text-white">
          {selectedDate ? `Transaksi ${formatSelectedDate(selectedDate)}` : 'Transaksi Bulan Terpilih'}
        </h3>
      </div>

      {#if displayedTransactions.length === 0}
        <div class="p-12 text-center">
          <div class="inline-flex p-4 bg-gray-800 rounded-full mb-4">
            <ReceiptIcon size={32} class="text-gray-600" />
          </div>
          <p class="text-gray-400 text-lg font-medium mb-2">
            {selectedDate ? 'Tidak ada transaksi pada tanggal ini' : 'Belum ada transaksi pada bulan ini'}
          </p>
          {#if selectedDate}
            <p class="text-gray-600 text-sm">Klik tombol Reset di filter tanggal untuk melihat semua transaksi bulanan.</p>
          {:else}
            <p class="text-gray-600 text-sm">Tekan <kbd class="px-2 py-1 bg-gray-800 rounded text-xs">Ctrl+N</kbd> untuk menambah transaksi pertama</p>
          {/if}
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
                      <div class="w-2 h-2 rounded-full {getCategoryColor(getDisplayCategory(transaction))}"></div>
                    </div>

                    <div class="flex-1 min-w-0">
                      <p class="text-white font-semibold truncate">
                        {isTransfer(transaction)
                          ? (transaction.description || 'Transfer')
                          : (transaction.description || transaction.category || 'Biaya Umum')}
                      </p>
                      <div class="flex items-center gap-2 mt-0.5">
                        <span class="text-xs text-gray-500">{formatTime(transaction.date)}</span>
                        <span class="text-xs text-gray-700">•</span>
                        <span class="text-xs text-gray-500">{getAccountName(transaction.account_id)}</span>
                        <span class="text-xs text-gray-700">|</span>
                        <span class="text-xs text-gray-500">{getDisplayCategory(transaction)}</span>
                        {#if isTransfer(transaction)}
                          <span class="text-xs text-blue-400">({getTransferLabel(transaction)})</span>
                        {/if}
                      </div>
                    </div>

                    <div class="flex items-center gap-3">
                      <p class="text-lg font-mono {transaction.amount >= 0 ? 'text-green-400' : 'text-red-400'}" style="font-feature-settings: 'tnum';">
                        {transaction.amount >= 0 ? '+' : ''}{formatCurrency(transaction.amount)}
                      </p>
                      {#if !isTransfer(transaction)}
                        <button
                          on:click={() => dispatch('edit', { transaction })}
                          class="opacity-0 group-hover:opacity-100 p-2 hover:bg-blue-500/10 hover:scale-110 rounded-lg text-blue-400 transition-all duration-200"
                          title="Edit"
                        >
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                          </svg>
                        </button>
                      {/if}
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

  <div class="hidden xl:flex xl:w-80 2xl:w-96 bg-gray-900 border-l border-gray-800 flex-col h-full flex-shrink-0">
    <div class="p-4 xl:p-6 pb-4 flex-shrink-0">
      <h3 class="text-lg font-bold text-white">Statistik</h3>
    </div>

    <div class="flex-1 px-4 xl:px-6 pb-4 xl:pb-6 overflow-y-auto">
      <div class="space-y-3">
      <div class="bg-gray-800/50 rounded-xl p-5 border border-gray-700/50">
        <p class="text-gray-400 text-xs font-medium mb-4 uppercase tracking-wider">
          {selectedDate ? 'Arus Kas Tanggal Dipilih' : 'Arus Kas Bulanan'}
        </p>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full bg-red-400"></div>
              <span class="text-sm text-gray-400">Pengeluaran</span>
            </div>
            <span class="text-lg font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(totalExpenseForStats)}</span>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full bg-green-400"></div>
              <span class="text-sm text-gray-400">Pemasukan</span>
            </div>
            <span class="text-lg font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(totalIncomeForStats)}</span>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">{selectedDate ? 'Biaya Tanggal Dipilih' : 'Rata-rata Biaya Harian'}</p>
          <p class="text-xl font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(dailyAverage)}</p>
        </div>

        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">Jml Transaksi</p>
          <p class="text-xl font-mono text-white" style="font-feature-settings: 'tnum';">{transactionCount}</p>
        </div>

        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">Rasio Menabung</p>
          <p class="text-xl font-mono {savingRate >= 0 ? 'text-green-300' : 'text-red-300'}" style="font-feature-settings: 'tnum';">
            {formatPercent(savingRate)}
          </p>
        </div>

        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">Top Kategori Biaya</p>
          {#if topExpenseCategoryForView}
            <p class="text-sm text-gray-200 truncate">{topExpenseCategoryForView[0]}</p>
            <p class="text-base font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(topExpenseCategoryForView[1])}</p>
          {:else}
            <p class="text-sm text-gray-500">Belum ada data</p>
          {/if}
        </div>
      </div>

      {#if !selectedDate}
        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50 space-y-3">
          <p class="text-gray-500 text-xs uppercase tracking-wider">Perbandingan Vs Bulan Lalu</p>
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-400">Pengeluaran</span>
            <span class="text-sm font-mono {expenseMoMChange !== null && expenseMoMChange <= 0 ? 'text-green-300' : 'text-red-300'}">
              {formatPercent(expenseMoMChange)}
            </span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-400">Pemasukan</span>
            <span class="text-sm font-mono {incomeMoMChange !== null && incomeMoMChange >= 0 ? 'text-green-300' : 'text-red-300'}">
              {formatPercent(incomeMoMChange)}
            </span>
          </div>
        </div>

        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <div class="flex items-center justify-between mb-3">
            <p class="text-gray-500 text-xs uppercase tracking-wider">Tren Kekayaan Bersih (6 Bulan)</p>
            {#if !isLoadingStats}
              <span class="text-xs font-mono {netWorthChange !== null && netWorthChange >= 0 ? 'text-green-300' : 'text-red-300'}">
                {formatPercent(netWorthChange)}
              </span>
            {/if}
          </div>

          {#if isLoadingStats}
            <p class="text-xs text-gray-500">Memuat statistik...</p>
          {:else if netWorthSeries.length > 0}
            <div class="space-y-2">
              <svg viewBox="0 0 260 88" class="w-full h-20">
                <polyline
                  fill="none"
                  stroke="rgb(59 130 246)"
                  stroke-width="3"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  points={netWorthPoints}
                />
              </svg>
              <div class="flex items-center justify-between">
                <span class="text-xs text-gray-500">{netWorthSeries[0].month}</span>
                <span class="text-sm font-mono text-white" style="font-feature-settings: 'tnum';">
                  {formatCurrency(netWorthCurrent)}
                </span>
                <span class="text-xs text-gray-500">{netWorthSeries[netWorthSeries.length - 1].month}</span>
              </div>
            </div>
          {:else}
            <p class="text-xs text-gray-500">Belum ada data kekayaan bersih.</p>
          {/if}
        </div>
      {/if}
      </div>
    </div>
  </div>
</div>
