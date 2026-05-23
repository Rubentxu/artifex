<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'ghost' | 'icon' | 'full';
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    onclick?: (event: MouseEvent) => void;
    class?: string;
    children?: Snippet;
    icon?: Snippet;
  }

  let {
    variant = 'primary',
    disabled = false,
    type = 'button',
    onclick,
    class: className = '',
    children,
    icon,
  }: Props = $props();

  const baseClasses = 'px-4 py-2 rounded-sm font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-[var(--color-neon)]/50';

  const variantClasses: Record<string, string> = {
    primary: 'bg-[var(--color-neon)] hover:shadow-[var(--glow-neon)] text-[#0A0A0F] font-bold inline-flex items-center gap-2',
    secondary: 'bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 inline-flex items-center gap-2',
    ghost: 'hover:bg-[var(--color-surface)]',
    icon: 'p-1.5 rounded-sm hover:bg-[var(--color-surface)]',
    full: 'w-full flex items-center justify-center gap-2',
  };

  const disabledClasses = 'opacity-50 cursor-not-allowed';

  const classes = $derived([
    baseClasses,
    variantClasses[variant],
    disabled ? disabledClasses : '',
    className,
  ].filter(Boolean).join(' '));

  function handleClick(event: MouseEvent) {
    if (disabled) {
      event.preventDefault();
      event.stopPropagation();
      return;
    }
    onclick?.(event);
  }
</script>

<button
  {type}
  class={classes}
  disabled={disabled}
  onclick={handleClick}
>
  {#if icon}
    {@render icon()}
  {/if}
  {#if children}
    {@render children()}
  {/if}
</button>
