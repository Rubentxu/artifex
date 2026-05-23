<script lang="ts">
  interface Props {
    label: string;
    value?: string;
    placeholder?: string;
    rows?: number;
    required?: boolean;
    id?: string;
    error?: string | null;
  }

  let {
    label,
    value = $bindable(''),
    placeholder = '',
    rows = 3,
    required = false,
    id,
    error = null,
  }: Props = $props();

  const textareaId = $derived(id ?? `textarea-${label.toLowerCase().replace(/\s+/g, '-')}`);
</script>

<div class="space-y-1">
  <label for={textareaId} class="block text-sm font-medium text-[var(--color-text)]">
    {label}
    {#if required}
      <span class="text-red-400">*</span>
    {/if}
  </label>
  <textarea
    id={textareaId}
    bind:value
    {placeholder}
    {rows}
    class="w-full px-3 py-2 bg-[var(--color-panel)] border rounded-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] resize-none transition-all duration-200 {error ? 'border-[var(--color-destructive)]' : 'border-[var(--color-surface)]'}"
  ></textarea>
  {#if error}
    <p class="text-sm text-red-400">{error}</p>
  {/if}
</div>
