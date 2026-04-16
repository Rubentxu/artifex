<script lang="ts">
  import type { ModelProfileDto, RoutingRuleDto } from '$lib/api/model-config';

  interface Props {
    operationType: string;
    profiles: ModelProfileDto[];
    rules: RoutingRuleDto[];
    onSelect?: (operationType: string, profileId: string) => void;
  }

  let { operationType, profiles, rules, onSelect }: Props = $props();

  // Find current rule for this operation type
  const currentRule = $derived(rules.find(r => r.operation_type === operationType));
  let selectedProfileId = $state<string>(currentRule?.default_profile_id ?? '');

  // Update when rule changes
  $effect(() => {
    selectedProfileId = currentRule?.default_profile_id ?? '';
  });

  function handleChange(e: Event) {
    const profileId = (e.target as HTMLSelectElement).value;
    selectedProfileId = profileId;
    onSelect?.(operationType, profileId);
  }

  // Get profiles that support the capability
  // Note: Rust sends lowercase capability names (e.g., "imagegen", "textcomplete")
  const capabilityMap: Record<string, string[]> = {
    'imagegen.txt2img': ['imagegen'],
    'textgen.complete': ['textcomplete'],
    'tts.npc_line': ['tts'],
    'audio.generate': ['audiogen'],
  };

  const relevantProfiles = $derived(
    profiles.filter(p => {
      const caps = capabilityMap[operationType] ?? [];
      return caps.some(cap => p.capabilities.includes(cap)) && p.enabled;
    })
  );
</script>

<div class="space-y-2">
  <label for="model-{operationType}" class="block text-sm font-medium text-[var(--color-text)]">
    {operationType}
  </label>

  {#if relevantProfiles.length === 0}
    <p class="text-sm text-[var(--color-text-muted)]">No profiles available for {operationType}</p>
  {:else}
    <select
      id="model-{operationType}"
      value={selectedProfileId}
      onchange={handleChange}
      class="w-full px-3 py-2 bg-[var(--color-panel)] border border-[ var(--color-surface)] rounded-lg text-[ var(--color-text)] focus:outline-none focus:border-[var(--color-accent)]"
    >
      <option value="" disabled>Select a model...</option>
      {#each relevantProfiles as profile (profile.id)}
        <option value={profile.id}>
          {profile.display_name} ({profile.provider_name})
        </option>
      {/each}
    </select>
  {/if}
</div>
