<script lang="ts">
  import type { AssetResponse } from '$lib/types/asset';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    material: AssetResponse;
  }

  let { material }: Props = $props();

  // Extract map paths from metadata
  interface MapPaths {
    basecolor: string | null;
    normal: string | null;
    roughness: string | null;
    metalness: string | null;
    height: string | null;
  }

  function getMapPaths(material: AssetResponse): MapPaths {
    const metadata = material.metadata as Record<string, Record<string, string> | null> | null;
    const maps = metadata?.maps;
    return {
      basecolor: maps?.basecolor ?? null,
      normal: maps?.normal ?? null,
      roughness: maps?.roughness ?? null,
      metalness: maps?.metalness ?? null,
      height: maps?.height ?? null,
    };
  }

  let mapPaths = $derived(getMapPaths(material));

  const mapLabels: Record<keyof MapPaths, string> = {
    basecolor: 'Base Color',
    normal: 'Normal',
    roughness: 'Roughness',
    metalness: 'Metalness',
    height: 'Height',
  };

  const mapKeys = ['basecolor', 'normal', 'roughness', 'metalness', 'height'] as const;
</script>

<div class="space-y-4">
  <h3 class="text-lg font-semibold">PBR Material Maps</h3>

  <div class="grid grid-cols-5 gap-2">
    {#each mapKeys as mapKey}
      {@const mapPath = mapPaths[mapKey]}
      <div class="flex flex-col items-center">
        <div class="w-full aspect-square rounded-lg border border-[var(--color-surface)] overflow-hidden bg-[var(--color-surface)]">
          {#if mapPath}
            <img
              src={convertFileSrc(mapPath)}
              alt={mapLabels[mapKey]}
              class="w-full h-full object-cover"
            />
          {:else}
            <div class="w-full h-full flex items-center justify-center text-[var(--color-text-muted)]">
              <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
            </div>
          {/if}
        </div>
        <span class="mt-1 text-xs text-[var(--color-text-muted)]">{mapLabels[mapKey]}</span>
      </div>
    {/each}
  </div>
</div>
