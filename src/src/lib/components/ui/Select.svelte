<script lang="ts">
  interface Props {
    label: string;
    value?: string | number;
    options?: Array<{ value: string | number; label: string }>;
    required?: boolean;
    id?: string;
    error?: string | null;
    placeholder?: string;
  }

  let {
    label,
    value = $bindable(''),
    options = [],
    required = false,
    id,
    error = null,
    placeholder,
  }: Props = $props();

  const selectId = $derived(id ?? `select-${label.toLowerCase().replace(/\s+/g, '-')}`);
</script>

<div class="space-y-1">
  <label for={selectId} class="block text-sm font-medium text-[var(--color-text)]">
    {label}
    {#if required}
      <span class="text-red-400">*</span>
    {/if}
  </label>
  <select
    id={selectId}
    bind:value
    class="w-full px-3 py-2 bg-[var(--color-panel)] border rounded-sm text-[var(--color-text)] focus:outline-none focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] transition-all duration-200 {error ? 'border-[var(--color-destructive)]' : 'border-[var(--color-surface)]'}"
  >
    {#if placeholder}
      <option value="" disabled>{placeholder}</option>
    {/if}
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
  {#if error}
    <p class="text-sm text-red-400">{error}</p>
  {/if}
</div>
