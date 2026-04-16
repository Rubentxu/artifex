<script lang="ts">
  import type { ProviderDto } from '$lib/api/model-config';

  interface Props {
    providers: ProviderDto[];
    onToggle?: (providerId: string, enabled: boolean) => void;
  }

  let { providers, onToggle }: Props = $props();
</script>

<div class="space-y-3">
  <h3 class="text-lg font-semibold text-[var(--color-text)]">Providers</h3>

  {#if providers.length === 0}
    <p class="text-[var(--color-text-muted)] text-sm">No providers registered.</p>
  {:else}
    <div class="space-y-2">
      {#each providers as provider (provider.id)}
        <div class="flex items-center justify-between p-3 bg-[var(--color-surface)] rounded-lg">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-[var(--color-panel)] flex items-center justify-center">
              <svg class="w-5 h-5 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <div>
              <div class="font-medium text-[var(--color-text)]">{provider.name}</div>
              <div class="text-xs text-[var(--color-text-muted)]">
                {provider.kind} • {provider.supported_capabilities.join(', ')}
              </div>
            </div>
          </div>

          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={provider.enabled}
              onchange={(e) => onToggle?.(provider.id, (e.target as HTMLInputElement).checked)}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-[var(--color-panel)] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
          </label>
        </div>
      {/each}
    </div>
  {/if}
</div>
