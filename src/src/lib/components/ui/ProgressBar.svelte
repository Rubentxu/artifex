<script lang="ts">
  interface Props {
    value?: number;
    max?: number;
    class?: string;
  }

  let { value = 0, max = 100, class: className = '' }: Props = $props();

  const percentage = $derived(Math.min(100, Math.max(0, (value / max) * 100)));
</script>

<div class="w-full bg-[var(--color-surface)] rounded h-2 overflow-hidden {className}">
  <div
    class="h-full bg-[var(--color-cyan)] shadow-[var(--glow-cyan)] transition-all duration-300 ease-out"
    style="width: {percentage}%"
    role="progressbar"
    aria-valuenow={value}
    aria-valuemin={0}
    aria-valuemax={max}
  ></div>
</div>
