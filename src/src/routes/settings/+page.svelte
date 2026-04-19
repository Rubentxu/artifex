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
  import { identityStore } from '$lib/stores/identity';
  import type { UsageEntry } from '$lib/types';

  let providers = $state<ProviderDto[]>([]);
  let profiles = $state<ModelProfileDto[]>([]);
  let rules = $state<RoutingRuleDto[]>([]);
  let templates = $state<PromptTemplateDto[]>([]);

  let loading = $state(true);
  let error = $state<string | null>(null);

  // Profile form state
  let profileDisplayName = $state('');
  let profileEmail = $state('');
  let profileSaving = $state(false);
  let profileSaveSuccess = $state(false);

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
    await identityStore.loadIdentity();
    await identityStore.loadUsage();
    // Initialize profile form from store
    if ($identityStore.user) {
      profileDisplayName = $identityStore.user.display_name;
      profileEmail = $identityStore.user.email || '';
    }
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

  async function handleProfileSave() {
    profileSaving = true;
    profileSaveSuccess = false;
    try {
      await identityStore.updateProfile(profileDisplayName, profileEmail || null, null);
      profileSaveSuccess = true;
      setTimeout(() => profileSaveSuccess = false, 3000);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      profileSaving = false;
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

  function getUsageForOperation(usage: UsageEntry[], operation: string): UsageEntry | undefined {
    const currentPeriod = new Date().toISOString().slice(0, 7); // YYYY-MM
    return usage.find(u => u.operation_type === operation && u.period === currentPeriod);
  }

  function getUsagePercentage(usage: UsageEntry): number {
    if (usage.limit === 0 || usage.limit === 2147483647) return 0;
    return Math.min(100, (usage.count / usage.limit) * 100);
  }

  function getNextResetDate(): string {
    const now = new Date();
    const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
    return nextMonth.toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' });
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
        <!-- Profile Section -->
        <section class="p-6 bg-[var(--color-panel)] rounded-xl space-y-4">
          <h3 class="text-lg font-semibold text-[var(--color-text)]">Profile</h3>
          <div class="grid gap-4 md:grid-cols-2">
            <div class="space-y-2">
              <label for="displayName" class="text-sm font-medium text-[var(--color-text)]">Display Name</label>
              <input
                id="displayName"
                type="text"
                bind:value={profileDisplayName}
                class="w-full px-3 py-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
              />
            </div>
            <div class="space-y-2">
              <label for="email" class="text-sm font-medium text-[var(--color-text)]">Email</label>
              <input
                id="email"
                type="email"
                bind:value={profileEmail}
                class="w-full px-3 py-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
              />
            </div>
          </div>
          <div class="flex items-center gap-4">
            <button
              onclick={handleProfileSave}
              disabled={profileSaving}
              class="px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 disabled:opacity-50 rounded-lg transition-colors font-medium"
            >
              {profileSaving ? 'Saving...' : 'Save Profile'}
            </button>
            {#if profileSaveSuccess}
              <span class="text-green-400 text-sm">Profile saved!</span>
            {/if}
          </div>
        </section>

        <!-- Tier & License Section -->
        <section class="p-6 bg-[var(--color-panel)] rounded-xl space-y-4">
          <h3 class="text-lg font-semibold text-[var(--color-text)]">Tier & License</h3>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <div class="px-4 py-2 rounded-lg font-bold text-sm uppercase
                {$identityStore.tier === 'pro'
                  ? 'bg-purple-500/20 text-purple-400 border border-purple-500/50'
                  : 'bg-gray-500/20 text-gray-400 border border-gray-500/50'
                }">
                {$identityStore.tier}
              </div>
              <div>
                <p class="text-sm text-[var(--color-text)]">
                  {$identityStore.tier === 'pro' ? 'Pro Tier - All features unlocked' : 'Free Tier - Limited features'}
                </p>
              </div>
            </div>
            {#if $identityStore.tier !== 'pro'}
              <a
                href="https://artifex.example.com/upgrade"
                target="_blank"
                rel="noopener noreferrer"
                class="px-4 py-2 bg-purple-500 hover:bg-purple-400 rounded-lg transition-colors font-medium text-white"
              >
                Upgrade to Pro
              </a>
            {/if}
          </div>
        </section>

        <!-- Usage Stats Section -->
        <section class="p-6 bg-[var(--color-panel)] rounded-xl space-y-4">
          <h3 class="text-lg font-semibold text-[var(--color-text)]">Usage Stats</h3>
          <div class="space-y-4">
            {#if $identityStore.tier === 'pro'}
              <div class="flex items-center gap-2 text-purple-400">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                <span>Unlimited usage on Pro tier</span>
              </div>
            {:else}
              <!-- Image Generation -->
              {@const imageUsage = getUsageForOperation($identityStore.usage, 'image_generate')}
              <div class="space-y-2">
                <div class="flex justify-between text-sm">
                  <span class="text-[var(--color-text)]">Image Generation</span>
                  <span class="text-[var(--color-text-muted)]">
                    {imageUsage ? `${imageUsage.count} / ${imageUsage.limit}` : '0 / 50'} this month
                  </span>
                </div>
                <div class="w-full h-2 bg-[var(--color-surface)] rounded-full overflow-hidden">
                  <div
                    class="h-full bg-blue-500 transition-all duration-300"
                    style="width: {imageUsage ? getUsagePercentage(imageUsage) : 0}%"
                  ></div>
                </div>
              </div>

              <!-- Audio Generation -->
              {@const audioUsage = getUsageForOperation($identityStore.usage, 'audio_generate')}
              <div class="space-y-2">
                <div class="flex justify-between text-sm">
                  <span class="text-[var(--color-text)]">Audio Generation</span>
                  <span class="text-[var(--color-text-muted)]">
                    {audioUsage ? `${audioUsage.count} / ${audioUsage.limit}` : '0 / 20'} this month
                  </span>
                </div>
                <div class="w-full h-2 bg-[var(--color-surface)] rounded-full overflow-hidden">
                  <div
                    class="h-full bg-green-500 transition-all duration-300"
                    style="width: {audioUsage ? getUsagePercentage(audioUsage) : 0}%"
                  ></div>
                </div>
              </div>

              <p class="text-xs text-[var(--color-text-muted)]">
                Resets on {getNextResetDate()}
              </p>
            {/if}
          </div>
        </section>

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
