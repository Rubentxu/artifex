<script lang="ts">
  interface Props {
    label: string;
    type?: 'text' | 'number' | 'email' | 'password' | 'url' | 'range';
    value?: string | number;
    placeholder?: string;
    error?: string | null;
    required?: boolean;
    id?: string;
    maxlength?: number;
    min?: number;
    max?: number;
    step?: number;
  }

  let {
    label,
    type = 'text',
    value = $bindable(''),
    placeholder = '',
    error = null,
    required = false,
    id,
    maxlength,
    min,
    max,
    step,
  }: Props = $props();

  const inputId = $derived(id ?? `input-${label.toLowerCase().replace(/\s+/g, '-')}`);
</script>

<div class="space-y-1">
  <label for={inputId} class="block text-sm font-medium text-[var(--color-text)]">
    {label}
    {#if required}
      <span class="text-red-400">*</span>
    {/if}
  </label>
  <input
    {type}
    id={inputId}
    bind:value
    {placeholder}
    {maxlength}
    {min}
    {max}
    {step}
    class="w-full px-3 py-2 bg-[var(--color-panel)] border rounded-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] transition-all duration-200 {error ? 'border-[var(--color-destructive)]' : 'border-[var(--color-surface)]'}"
  />
  {#if error}
    <p class="text-sm text-red-400">{error}</p>
  {/if}
</div>
