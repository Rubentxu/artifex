import { describe, it, expect, beforeEach } from 'vitest';
import * as modelConfig from '$lib/api/model-config';
import { setBackend } from '$lib/api/model-config';
import { MemoryBackend } from '$lib/api/memory-backend';
import type { ProviderDto, ModelProfileDto, RoutingRuleDto, PromptTemplateDto } from '$lib/api/model-config';

describe('model-config API', () => {
  let backend: MemoryBackend;

  beforeEach(() => {
    backend = new MemoryBackend();
    setBackend(backend as import('$lib/api/storage-backend').StorageBackend);
  });

  // === Provider Types ===
  const mockProvider: ProviderDto = {
    id: 'replicate',
    name: 'Replicate',
    kind: 'replicate',
    base_url: 'https://api.replicate.com/v1',
    supported_capabilities: ['image_gen'],
    auth_type: 'api_key',
    enabled: true,
  };

  // === E-1: Provider toggle flow ===
  describe('setProviderEnabled', () => {
    it('calls set_provider_enabled with provider.id', async () => {
      backend.on('set_provider_enabled', ({ provider_id, enabled }: { provider_id: string; enabled: boolean }) => {
        expect(provider_id).toBe('replicate');
        expect(enabled).toBe(true);
        return undefined;
      });

      await modelConfig.setProviderEnabled('replicate', true);
    });

    it('uses lowercase provider.id for toggle', async () => {
      backend.on('set_provider_enabled', ({ provider_id }: { provider_id: string }) => {
        // Backend normalizes to lowercase, so frontend should pass lowercase
        expect(provider_id).toBe('replicate');
        return undefined;
      });

      await modelConfig.setProviderEnabled('replicate', true);
    });
  });

  // === E-2: API key save/test/delete flow ===
  describe('Credential operations use provider.id', () => {
    it('setCredential calls with provider.id', async () => {
      backend.on('set_credential', ({ provider_id, api_key }: { provider_id: string; api_key: string }) => {
        expect(provider_id).toBe('replicate');
        expect(api_key).toBe('test-api-key-123');
        return undefined;
      });

      await modelConfig.setCredential('replicate', 'test-api-key-123');
    });

    it('testProviderConnection calls with provider.id', async () => {
      backend.on('test_provider_connection', ({ provider_id }: { provider_id: string }) => {
        expect(provider_id).toBe('replicate');
        return true;
      });

      const result = await modelConfig.testProviderConnection('replicate');
      expect(result).toBe(true);
    });

    it('deleteCredential calls with provider.id', async () => {
      backend.on('delete_credential', ({ provider_id }: { provider_id: string }) => {
        expect(provider_id).toBe('replicate');
        return undefined;
      });

      await modelConfig.deleteCredential('replicate');
    });

    it('getCredentialStatus calls with provider.id', async () => {
      backend.on('get_credential_status', ({ provider_id }: { provider_id: string }) => {
        expect(provider_id).toBe('kie');
        return { provider_id: 'kie', has_credential: true };
      });

      const result = await modelConfig.getCredentialStatus('kie');
      expect(result.provider_id).toBe('kie');
    });

    it('getProvider calls with provider_id (lowercase)', async () => {
      backend.on('get_provider', ({ provider_id }: { provider_id: string }) => {
        expect(provider_id).toBe('kie');
        return { ...mockProvider, id: 'kie', name: 'Kie AI' };
      });

      const result = await modelConfig.getProvider('kie');
      expect(result).toBeTruthy();
      expect(result?.id).toBe('kie');
    });
  });

  // === E-3: Model selector tests ===
  describe('Model profiles and routing rules', () => {
    const mockProfiles: ModelProfileDto[] = [
      {
        id: 'profile-1',
        provider_name: 'replicate',
        model_id: 'black-forest-labs/flux-1.1-pro',
        display_name: 'FLUX 1.1 Pro (Replicate)',
        capabilities: ['image_gen'],
        enabled: true,
        pricing_tier: 'standard',
        config: {},
      },
      {
        id: 'profile-2',
        provider_name: 'fal',
        model_id: 'fal-ai/flux-1-dev',
        display_name: 'FLUX.1 Dev (Fal)',
        capabilities: ['image_gen'],
        enabled: true,
        pricing_tier: 'standard',
        config: {},
      },
      {
        id: 'profile-3',
        provider_name: 'together',
        model_id: 'meta-llama/Meta-Llama-3-70B',
        display_name: 'Llama 3 (Together)',
        capabilities: ['text_complete'],
        enabled: true,
        pricing_tier: 'standard',
        config: {},
      },
    ];

    const mockRules: RoutingRuleDto[] = [
      {
        id: 'rule-1',
        operation_type: 'imagegen.txt2img',
        default_profile_id: 'profile-1',
        fallback_profile_ids: ['profile-2'],
      },
      {
        id: 'rule-2',
        operation_type: 'textgen.complete',
        default_profile_id: 'profile-3',
        fallback_profile_ids: [],
      },
    ];

    it('listModelProfiles returns profiles with provider_name', async () => {
      backend.on('list_model_profiles', () => mockProfiles);

      const result = await modelConfig.listModelProfiles();

      expect(result).toHaveLength(3);
      expect(result[0].provider_name).toBe('replicate');
    });

    it('listRoutingRules returns rules with fallback chain', async () => {
      backend.on('list_routing_rules', () => mockRules);

      const result = await modelConfig.listRoutingRules();

      expect(result).toHaveLength(2);
      const imageRule = result.find(r => r.operation_type === 'imagegen.txt2img');
      expect(imageRule?.fallback_profile_ids).toContain('profile-2');
    });

    it('getRoutingRule finds rule by operation type', async () => {
      backend.on('list_routing_rules', () => mockRules);

      const result = await modelConfig.getRoutingRule('imagegen.txt2img');

      expect(result).toBeTruthy();
      expect(result?.default_profile_id).toBe('profile-1');
      expect(result?.fallback_profile_ids).toContain('profile-2');
    });

    it('capability filter uses lowercase values', async () => {
      backend.on('list_model_profiles', () => mockProfiles);
      backend.on('list_routing_rules', () => mockRules);

      const profiles = await modelConfig.listModelProfiles();

      // Profiles with image_gen capability
      const imageProfiles = profiles.filter(p => p.capabilities.includes('image_gen'));
      expect(imageProfiles).toHaveLength(2);
    });
  });

  // === E-4: Prompt template CRUD ===
  describe('Prompt template CRUD', () => {
    const mockTemplates: PromptTemplateDto[] = [
      {
        id: 'template-1',
        name: 'NPC Dialogue',
        template_text: 'You are {{character}}, in a {{setting}}. Say: {{dialogue}}',
        variables: ['character', 'setting', 'dialogue'],
      },
      {
        id: 'template-2',
        name: 'Scene Description',
        template_text: 'Describe {{scene}} in the style of {{style}}',
        variables: ['scene', 'style'],
      },
    ];

    it('listPromptTemplates returns templates', async () => {
      backend.on('list_prompt_templates', () => mockTemplates);

      const result = await modelConfig.listPromptTemplates();

      expect(result).toHaveLength(2);
      expect(result[0].name).toBe('NPC Dialogue');
    });

    it('createPromptTemplate calls with name and template_text', async () => {
      backend.on('create_prompt_template', ({ name, template_text }: { name: string; template_text: string }) => {
        expect(name).toBe('New Template');
        expect(template_text).toBe('Hello {{name}}!');
        return {
          id: 'template-3',
          name: 'New Template',
          template_text: 'Hello {{name}}!',
          variables: ['name'],
        };
      });

      const result = await modelConfig.createPromptTemplate('New Template', 'Hello {{name}}!');

      expect(result.name).toBe('New Template');
      expect(result.variables).toContain('name');
    });

    it('deletePromptTemplate calls with id', async () => {
      backend.on('delete_prompt_template', ({ id }: { id: string }) => {
        expect(id).toBe('template-1');
        return undefined;
      });

      await modelConfig.deletePromptTemplate('template-1');
    });
  });

  // === Provider listing ===
  describe('listProviders', () => {
    it('returns providers with id field', async () => {
      const mockProviders: ProviderDto[] = [
        { ...mockProvider, id: 'replicate', name: 'Replicate' },
        { ...mockProvider, id: 'fal', name: 'Fal' },
        { ...mockProvider, id: 'kie', name: 'Kie AI' },
      ];

      backend.on('list_providers', () => mockProviders);

      const result = await modelConfig.listProviders();

      expect(result).toHaveLength(3);
      expect(result.map(p => p.id)).toEqual(['replicate', 'fal', 'kie']);
    });
  });
});
