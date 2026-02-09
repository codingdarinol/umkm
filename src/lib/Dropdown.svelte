<script lang="ts">
  import { createEventDispatcher, onDestroy, tick } from 'svelte';
  import { ChevronDown } from 'lucide-svelte';

  export let value: string | number;
  export let options: Array<{ value: string | number; label: string }>;
  export let placeholder = 'Select...';
  export let icon: any = null;
  export let disabled = false;

  const dispatch = createEventDispatcher();

  let isOpen = false;
  let dropdownRef: HTMLDivElement;
  let menuRef: HTMLDivElement;
  let menuStyle = '';
  let listenersAttached = false;

  function portal(node: HTMLElement) {
    if (typeof document === 'undefined') return {};
    document.body.appendChild(node);
    return {
      destroy() {
        if (node.parentNode) {
          node.parentNode.removeChild(node);
        }
      }
    };
  }

  $: selectedOption = options.find(opt => opt.value === value);

  async function openDropdown() {
    if (disabled) return;
    isOpen = true;
    await tick();
    updateMenuPosition();
    attachListeners();
  }

  function closeDropdown() {
    isOpen = false;
    detachListeners();
  }

  function toggleDropdown() {
    if (disabled) return;
    if (isOpen) {
      closeDropdown();
    } else {
      openDropdown();
    }
  }

  function selectOption(option: typeof options[0]) {
    value = option.value;
    closeDropdown();
    dispatch('change', { value: option.value });
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as Node;
    if (dropdownRef?.contains(target) || menuRef?.contains(target)) {
      return;
    }
    closeDropdown();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeDropdown();
    }
  }

  function updateMenuPosition() {
    if (!dropdownRef || !menuRef) return;

    const rect = dropdownRef.getBoundingClientRect();
    const menuHeight = menuRef.offsetHeight;
    const spaceBelow = window.innerHeight - rect.bottom - 8;
    const spaceAbove = rect.top - 8;
    const openUp = spaceBelow < menuHeight && spaceAbove > spaceBelow;
    const top = openUp ? rect.top - menuHeight - 4 : rect.bottom + 4;
    const clampedTop = Math.max(8, Math.min(top, window.innerHeight - menuHeight - 8));

    menuStyle = `top:${clampedTop}px;left:${rect.left}px;width:${rect.width}px;`;
  }

  function attachListeners() {
    if (listenersAttached) return;
    window.addEventListener('resize', updateMenuPosition);
    window.addEventListener('scroll', updateMenuPosition, true);
    listenersAttached = true;
  }

  function detachListeners() {
    if (!listenersAttached) return;
    window.removeEventListener('resize', updateMenuPosition);
    window.removeEventListener('scroll', updateMenuPosition, true);
    listenersAttached = false;
  }

  onDestroy(() => {
    detachListeners();
  });
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

<div bind:this={dropdownRef} class="relative">
  <button
    type="button"
    on:click={toggleDropdown}
    class="w-full px-3 py-2.5 bg-gray-800 border-2 border-gray-700 rounded-lg text-white text-sm font-semibold focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-purple-500 hover:bg-gray-750 hover:border-gray-600 transition-all cursor-pointer flex items-center justify-between gap-2 {disabled ? 'opacity-50 cursor-not-allowed' : ''}"
    disabled={disabled}
  >
    <div class="flex items-center gap-2 flex-1 min-w-0">
      {#if icon}
        <svelte:component this={icon} size={16} class="text-gray-500 flex-shrink-0" />
      {/if}
      <span class="truncate">{selectedOption?.label || placeholder}</span>
    </div>
    <ChevronDown size={16} class="text-gray-400 flex-shrink-0 transition-transform {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div
      bind:this={menuRef}
      use:portal
      class="fixed z-[60] bg-gray-800 border-2 border-gray-700 rounded-lg shadow-2xl max-h-72 overflow-y-auto overscroll-contain"
      style={menuStyle}
    >
      {#each options as option}
        <button
          type="button"
          on:click={() => selectOption(option)}
          class="w-full px-3 py-2.5 text-left text-sm text-white hover:bg-gray-700 transition-colors flex items-center gap-2 {option.value === value ? 'bg-gray-700/50' : ''}"
        >
          <span class="truncate">{option.label}</span>
          {#if option.value === value}
            <div class="ml-auto w-1.5 h-1.5 rounded-full bg-purple-500 flex-shrink-0"></div>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .max-h-72 {
    max-height: 18rem;
  }

  .max-h-72::-webkit-scrollbar {
    width: 8px;
  }
  
  .max-h-72::-webkit-scrollbar-track {
    background: #1f2937;
    border-radius: 4px;
  }
  
  .max-h-72::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
  }
  
  .max-h-72::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
</style>
