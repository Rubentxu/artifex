<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    onClose: () => void;
    title?: string;
    error?: string | null;
    width?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '4xl';
    showCloseButton?: boolean;
    bordered?: boolean;
    scrollBody?: boolean;
    header?: Snippet;
    default?: Snippet;
    footer?: Snippet;
  }

  let {
    open = $bindable(false),
    onClose,
    title = '',
    error = null,
    width = 'md',
    showCloseButton = true,
    bordered = false,
    scrollBody = false,
    header,
    default: body,
    footer,
  }: Props = $props();

  const widthClasses: Record<string, string> = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl',
    '2xl': 'max-w-2xl',
    '4xl': 'max-w-4xl',
  };

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && open) {
      onClose();
    }
  }

  function handleBackdropClick() {
    onClose();
  }

  function handleDialogClick(event: MouseEvent) {
    event.stopPropagation();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
    onclick={handleBackdropClick}
    role="dialog"
    aria-modal="true"
  >
    <div
      class="bg-[var(--color-panel)] rounded w-full {widthClasses[width]} {scrollBody ? 'flex flex-col max-h-[90vh]' : 'max-h-[90vh] overflow-auto'} {bordered ? 'border border-[var(--color-surface)] hover:shadow-[var(--glow-cyan)] transition-shadow duration-200' : ''}"
      onclick={handleDialogClick}
    >
      {#if title || header}
        <header class="px-6 py-4 border-b border-[var(--color-surface)] flex items-center justify-between {scrollBody ? 'shrink-0' : ''}">
          {#if header}
            {@render header()}
          {:else}
            <h2 class="text-lg font-semibold text-[var(--color-text)]">{title}</h2>
          {/if}
          {#if showCloseButton}
            <button
              onclick={onClose}
              class="p-1.5 rounded-sm hover:bg-[var(--color-surface)] transition-colors text-[var(--color-muted)]"
              aria-label="Close dialog"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          {/if}
        </header>
      {/if}

      <main class="px-6 py-4 {scrollBody ? 'flex-1 overflow-y-auto' : ''}">
        {#if error}
          <div class="mb-4 p-3 bg-[var(--color-destructive)]/20 border border-[var(--color-destructive)]/50 rounded-sm text-[var(--color-destructive)] text-sm">
            {error}
          </div>
        {/if}
        {#if body}
          {@render body()}
        {/if}
      </main>

      {#if footer}
        <footer class="px-6 py-4 border-t border-[var(--color-surface)] {scrollBody ? 'shrink-0' : ''}">
          {@render footer()}
        </footer>
      {/if}
    </div>
  </div>
{/if}
