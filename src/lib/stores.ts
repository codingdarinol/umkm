import { writable } from 'svelte/store';

export interface CurrencySettings {
  code: string;
  symbol: string;
  position: 'before' | 'after';
  locale: string;
}

const defaultCurrency: CurrencySettings = {
  code: 'IDR',
  symbol: 'Rp',
  position: 'before',
  locale: 'id-ID'
};

function loadCurrency(): CurrencySettings {
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('spent_currency');
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch (e) {
        console.error('Failed to parse stored currency:', e);
      }
    }
  }
  return defaultCurrency;
}

export const currencySettings = writable<CurrencySettings>(loadCurrency());

currencySettings.subscribe(value => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('spent_currency', JSON.stringify(value));
  }
});

export const currencyOptions = [
  { code: 'IDR', symbol: 'Rp', name: 'Indonesian Rupiah', position: 'before' as const, locale: 'id-ID' },
  { code: 'USD', symbol: '$', name: 'US Dollar', position: 'before' as const, locale: 'en-US' },
];

export function formatCurrency(cents: number, settings: CurrencySettings): string {
  const dollars = Math.abs(cents) / 100;
  
  const formatted = new Intl.NumberFormat(settings.locale, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(dollars);
  
  if (settings.position === 'before') {
    return `${settings.symbol}${formatted}`;
  } else {
    return `${formatted} ${settings.symbol}`;
  }
}
