<script lang="ts">
  import { onMount } from 'svelte';
  import ProviderList from '$lib/components/ProviderList.svelte';
  import ApiKeyInput from '$lib/components/ApiKeyInput.svelte';
  import ModelSelector from '$lib/components/ModelSelector.svelte';
  import PromptTemplateEditor from '$lib/components/PromptTemplateEditor.svelte';
  import {
    listProviders,
    listModelProfiles,
    listRoutingRules,
    listPromptTemplates,
    setRoutingRule,
    setProviderEnabled,
    getCredentialStatus,
    type ProviderDto,
    type ModelProfileDto,
    type RoutingRuleDto,
    type PromptTemplateDto,
  } from '$lib/api/model-config';

  let providers = $state<ProviderDto[]>([]);
  let profiles = $state<ModelProfileDto[]>([]);
  let rules = $state<RoutingRuleDto[]>([]);
  let templates = $state<PromptTemplateDto[]>([]);

  let loading = $state(true);
  let error = $state<string | null>(null);

  // Track which providers have credentials configured (actual credential status from API)
  let configuredProviders = $state<Set<string>>(new Set());

  const operationTypes = [
    'imagegen.txt2img',
    'textgen.complete',
    'tts.npc_line',
    'audio.generate',
  ];

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    loading = true;
    error = null;

    try {
      const [providersData, profilesData, rulesData, templatesData] = await Promise.all([
        listProviders(),
        listModelProfiles(),
        listRoutingRules(),
        listPromptTemplates(),
      ]);

      providers = providersData;
      profiles = profilesData;
      rules = rulesData;
      templates = templatesData;

      // Get actual credential status for each provider using canonical provider_id
      const credentialStatuses = await Promise.all(
        providersData.map(p => getCredentialStatus(p.id))
      );
      configuredProviders = new Set(
        credentialStatuses
          .filter(s => s.has_credential)
          .map(s => s.provider_id)
      );
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function handleProviderToggle(providerId: string, enabled: boolean) {
    try {
      await setProviderEnabled(providerId, enabled);
      // Update the provider's enabled state locally
      providers = providers.map(p =>
        p.id === providerId ? { ...p, enabled } : p
      );
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleCredentialSave(providerId: string, _apiKey: string) {
    // Refresh credential status after saving
    try {
      const status = await getCredentialStatus(providerId);
      if (status.has_credential) {
        configuredProviders = new Set([...configuredProviders, providerId]);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleCredentialDelete(providerId: string) {
    // Refresh credential status after deleting
    configuredProviders.delete(providerId);
    configuredProviders = new Set(configuredProviders);
  }

  async function handleRoutingRuleChange(operationType: string, profileId: string) {
    try {
      const rule = rules.find(r => r.operation_type === operationType);
      if (rule) {
        await setRoutingRule(operationType, profileId, rule.fallback_profile_ids);
      } else {
        // Create new rule (Rust command will create if not exists)
        await setRoutingRule(operationType, profileId, []);
      }
      // Reload rules
      rules = await listRoutingRules();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <header class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-surface)]">
    <h1 class="text-2xl font-bold">Settings</h1>
    <button
      onclick={loadData}
      class="px-3 py-1.5 text-sm bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors"
    >
      Refresh
    </button>
  </header>

  <!-- Content -->
  <main class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <div class="text-[var(--color-text-muted)]">Loading...</div>
      </div>
    {:else if error}
      <div class="p-4 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400">
        {error}
      </div>
    {:else}
      <div class="max-w-4xl space-y-8">
        <!-- Providers Section -->
        <section class="p-6 bg-[var(--color-panel)] rounded-xl">
          <ProviderList {providers} onToggle={handleProviderToggle} />
        </section>

        <!-- API Keys Section -->
        <section class="p-6 bg-[var(--color-panel)] rounded-xl space-y-4">
          <h3 class="text-lg font-semibold text-[var(--color-text)]">API Keys</h3>
          <p class="text-sm text-[var(--color-text-muted)]">
            Configure API keys for your AI providers. Keys are stored securely in your system's keychain.
          </p>
          <div class="grid gap-4 md:grid-cols-2">
            {#each providers as provider (provider.id)}
              <ApiKeyInput
                providerId={provider.id}
                providerName={provider.name}
                hasCredential={configuredProviders.has(provider.id)}
                onSave={handleCredentialSave}
                onDelete={handleCredentialDelete}
              />
            {/each}
          </div>
        </section>

        <!-- Model Selection Section -->
        <section class="p-6 bg-[var(--color-panel)] rounded-xl space-y-4">
          <h3 class="text-lg font-semibold text-[var(--color-text)]">Model Selection</h3>
          <p class="text-sm text-[var(--color-text-muted)]">
            Choose the default model for each operation type. The router will use these settings to determine which model to use.
          </p>
          <div class="grid gap-6 md:grid-cols-2">
            {#each operationTypes as opType}
              <ModelSelector
                operationType={opType}
                {profiles}
                {rules}
                onSelect={handleRoutingRuleChange}
              />
            {/each}
          </div>
        </section>

        <!-- Prompt Templates Section -->
        <section class="p-6 bg-[var(--color-panel)] rounded-xl">
          <PromptTemplateEditor
            {templates}
            onUpdate={loadData}
          />
        </section>
      </div>
    {/if}
  </main>
</div>
