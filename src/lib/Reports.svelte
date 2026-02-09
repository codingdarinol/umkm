<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { currencySettings, formatCurrency as formatCurrencyHelper } from './stores';
  import Dropdown from './Dropdown.svelte';
  import { FileText, TrendingUp, Scale } from 'lucide-svelte';

  export let containerId: number | null = null;
  export let selectedMonth: string;
  export let availableMonths: string[];

  interface ProfitLossLine {
    category: string;
    total: number;
  }

  interface ProfitLossReport {
    start_date: string;
    end_date: string;
    income: ProfitLossLine[];
    expense: ProfitLossLine[];
    total_income: number;
    total_expense: number;
    net_income: number;
  }

  interface AccountBalance {
    id: number;
    name: string;
    account_type: string;
    opening_balance: number;
    balance: number;
    container_id: number;
    created_at: string;
  }

  interface BalanceSheetReport {
    as_of: string;
    assets: AccountBalance[];
    liabilities: AccountBalance[];
    equity: AccountBalance[];
    total_assets: number;
    total_liabilities: number;
    total_equity: number;
  }

  const dispatch = createEventDispatcher();

  let profitLoss: ProfitLossReport | null = null;
  let balanceSheet: BalanceSheetReport | null = null;
  let isLoading = false;
  let errorMessage = '';

  $: formatCurrency = (cents: number): string => {
    return formatCurrencyHelper(cents, $currencySettings);
  };

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

  function formatRange(start: string, end: string): string {
    const startDate = new Date(start);
    const endDate = new Date(end);
    return `${startDate.toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })} - ${endDate.toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })}`;
  }

  $: monthOptions = availableMonths.map(month => ({
    value: month,
    label: formatMonthLabel(month)
  }));

  async function loadReports() {
    if (!containerId || !selectedMonth) return;

    isLoading = true;
    errorMessage = '';
    try {
      profitLoss = await invoke<ProfitLossReport>('get_profit_and_loss_for_month', {
        containerId,
        month: selectedMonth,
      });
      balanceSheet = await invoke<BalanceSheetReport>('get_balance_sheet_for_month', {
        containerId,
        month: selectedMonth,
      });
    } catch (error) {
      console.error('Failed to load reports:', error);
      errorMessage = 'Gagal memuat laporan. Coba lagi.';
    } finally {
      isLoading = false;
    }
  }

  $: if (containerId && selectedMonth) {
    loadReports();
  }

  function handleMonthChange(event: CustomEvent) {
    dispatch('monthChange', { month: event.detail.value });
  }
</script>

<div class="p-4 lg:p-8 space-y-4 lg:space-y-6 h-full overflow-auto">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
    <div>
      <h2 class="text-2xl lg:text-3xl font-black text-white mb-1">Laporan</h2>
      <p class="text-xs lg:text-sm text-gray-500">Ringkasan laba rugi dan posisi keuangan</p>
    </div>
    <div class="flex items-center gap-2">
      <FileText size={18} class="text-gray-400 hidden sm:block" />
      <div class="min-w-[160px] sm:min-w-[200px]">
        <Dropdown
          value={selectedMonth}
          options={monthOptions}
          on:change={handleMonthChange}
        />
      </div>
    </div>
  </div>

  {#if isLoading}
    <div class="bg-gray-900 rounded-xl border border-gray-800 p-6 text-gray-400">Memuat laporan...</div>
  {:else if errorMessage}
    <div class="bg-red-500/10 border border-red-500/30 text-red-300 rounded-xl p-6">{errorMessage}</div>
  {:else}
    <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 lg:gap-6">
      <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg overflow-hidden">
        <div class="px-6 py-4 border-b border-gray-800 flex items-center gap-2">
          <TrendingUp size={18} class="text-green-400" />
          <h3 class="text-lg font-bold text-white">Laporan Laba Rugi</h3>
        </div>
        <div class="p-6 space-y-4">
          {#if profitLoss}
            <p class="text-xs text-gray-500">Periode: {formatRange(profitLoss.start_date, profitLoss.end_date)}</p>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
                <p class="text-xs text-gray-500 mb-1">Total Pendapatan</p>
                <p class="text-lg font-mono text-green-400">{formatCurrency(profitLoss.total_income)}</p>
              </div>
              <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
                <p class="text-xs text-gray-500 mb-1">Total Beban</p>
                <p class="text-lg font-mono text-red-400">{formatCurrency(profitLoss.total_expense)}</p>
              </div>
              <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
                <p class="text-xs text-gray-500 mb-1">Laba Bersih</p>
                <p class="text-lg font-mono {profitLoss.net_income >= 0 ? 'text-green-300' : 'text-red-300'}">
                  {formatCurrency(profitLoss.net_income)}
                </p>
              </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <h4 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Pendapatan</h4>
                <div class="space-y-2">
                  {#if profitLoss.income.length === 0}
                    <p class="text-sm text-gray-500">Belum ada pendapatan.</p>
                  {:else}
                    {#each profitLoss.income as line}
                      <div class="flex items-center justify-between bg-gray-800/40 rounded-lg px-3 py-2">
                        <span class="text-sm text-gray-200">{line.category}</span>
                        <span class="text-sm font-mono text-green-400">{formatCurrency(line.total)}</span>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
              <div>
                <h4 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Beban</h4>
                <div class="space-y-2">
                  {#if profitLoss.expense.length === 0}
                    <p class="text-sm text-gray-500">Belum ada beban.</p>
                  {:else}
                    {#each profitLoss.expense as line}
                      <div class="flex items-center justify-between bg-gray-800/40 rounded-lg px-3 py-2">
                        <span class="text-sm text-gray-200">{line.category}</span>
                        <span class="text-sm font-mono text-red-400">{formatCurrency(line.total)}</span>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg overflow-hidden">
        <div class="px-6 py-4 border-b border-gray-800 flex items-center gap-2">
          <Scale size={18} class="text-blue-400" />
          <h3 class="text-lg font-bold text-white">Laporan Posisi Keuangan</h3>
        </div>
        <div class="p-6 space-y-4">
          {#if balanceSheet}
            <p class="text-xs text-gray-500">Posisi per {new Date(balanceSheet.as_of).toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })}</p>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
                <p class="text-xs text-gray-500 mb-1">Total Aset</p>
                <p class="text-lg font-mono text-blue-300">{formatCurrency(balanceSheet.total_assets)}</p>
              </div>
              <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
                <p class="text-xs text-gray-500 mb-1">Total Liabilitas</p>
                <p class="text-lg font-mono text-orange-300">{formatCurrency(balanceSheet.total_liabilities)}</p>
              </div>
              <div class="bg-gray-800/50 rounded-lg p-3 border border-gray-700/50">
                <p class="text-xs text-gray-500 mb-1">Total Ekuitas</p>
                <p class="text-lg font-mono text-purple-300">{formatCurrency(balanceSheet.total_equity)}</p>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <h4 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Aset</h4>
                <div class="space-y-2">
                  {#if balanceSheet.assets.length === 0}
                    <p class="text-sm text-gray-500">Belum ada akun aset.</p>
                  {:else}
                    {#each balanceSheet.assets as account}
                      <div class="flex items-center justify-between bg-gray-800/40 rounded-lg px-3 py-2">
                        <span class="text-sm text-gray-200">{account.name}</span>
                        <span class="text-sm font-mono text-blue-300">{formatCurrency(account.balance)}</span>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
              <div>
                <h4 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Liabilitas</h4>
                <div class="space-y-2">
                  {#if balanceSheet.liabilities.length === 0}
                    <p class="text-sm text-gray-500">Belum ada akun liabilitas.</p>
                  {:else}
                    {#each balanceSheet.liabilities as account}
                      <div class="flex items-center justify-between bg-gray-800/40 rounded-lg px-3 py-2">
                        <span class="text-sm text-gray-200">{account.name}</span>
                        <span class="text-sm font-mono text-orange-300">{formatCurrency(account.balance)}</span>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
              <div>
                <h4 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Ekuitas</h4>
                <div class="space-y-2">
                  {#if balanceSheet.equity.length === 0}
                    <p class="text-sm text-gray-500">Belum ada akun ekuitas.</p>
                  {:else}
                    {#each balanceSheet.equity as account}
                      <div class="flex items-center justify-between bg-gray-800/40 rounded-lg px-3 py-2">
                        <span class="text-sm text-gray-200">{account.name}</span>
                        <span class="text-sm font-mono text-purple-300">{formatCurrency(account.balance)}</span>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
