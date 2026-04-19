/**
 * Mock Layer for AI Service Emulation in E2E Tests
 *
 * This module provides a MockBackend that intercepts Tauri IPC calls
 * and returns realistic fake data. It simulates the full job lifecycle
 * by emitting fake job-progress and job-completed events.
 *
 * ONLY active when explicitly enabled via window.__ARTIFEX_DEBUG__.enableMock()
 * Tree-shaken in production builds via import.meta.env.DEV
 */
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
import { MemoryBackend } from '$lib/api/memory-backend';
import type { StorageBackend } from '$lib/api/storage-backend';
import type { AssetResponse, AssetKind, CodeTemplate, CollectionResponse } from '$lib/types/asset';
import type { ProjectResponse, UserProfileDto, Tier, UsageEntry, QuotaResult } from '$lib/types';
import type { CodeAgentRequest } from '$lib/types';
import type { RoutingRuleDto, ModelProfileDto, PromptTemplateDto, CredentialStatusDto } from '$lib/api/model-config';
import * as projectsApi from '$lib/api/projects';
import * as assetsApi from '$lib/api/assets';
import * as agentApi from '$lib/api/agent';
import * as modelConfigApi from '$lib/api/model-config';

// ----------------------------------------------------------------------------
// Types
// ----------------------------------------------------------------------------

export type MockTier = 'free' | 'pro';
export type MockJobResult = 'success' | 'error';

interface MockState {
  isMockMode: boolean;
  mockTier: MockTier;
  mockJobResult: MockJobResult;
  mockData: Map<string, unknown>;
  callHistory: Array<{ command: string; args: unknown }>;
  originalBackends: Map<string, StorageBackend>;
  activeJobs: Map<string, ReturnType<typeof setTimeout>>;
  listeners: Map<string, UnlistenFn>;
}

// ----------------------------------------------------------------------------
// Fake Data Factories
// ----------------------------------------------------------------------------

function generateId(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 11)}`;
}

function fakeProject(overrides: Partial<ProjectResponse> = {}): ProjectResponse {
  return {
    id: generateId('proj'),
    name: 'Mock Project',
    path: '/mock/path',
    status: 'active',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    ...overrides,
  };
}

function fakeAsset(overrides: Partial<AssetResponse> = {}): AssetResponse {
  return {
    id: generateId('asset'),
    project_id: 'proj-mock',
    name: 'Mock Asset',
    kind: 'Image',
    file_path: null,
    metadata: null,
    file_size: 1024,
    width: 512,
    height: 512,
    duration_secs: null,
    sample_rate: null,
    created_at: new Date().toISOString(),
    tags: [],
    import_source: 'mock',
    collection_id: null,
    derived_from: null,
    ...overrides,
  };
}

const ASSET_KINDS: AssetKind[] = ['Image', 'Sprite', 'Tileset', 'Audio', 'Video', 'Animation', 'Code'];

function fakeAssetList(count: number, projectId: string): AssetResponse[] {
  return Array.from({ length: count }, (_, i) => {
    const kind = ASSET_KINDS[i % ASSET_KINDS.length];
    return fakeAsset({
      id: `asset-${i}`,
      project_id: projectId,
      name: `Mock ${kind} ${i + 1}`,
      kind,
      width: kind === 'Image' || kind === 'Sprite' ? 256 + i * 32 : null,
      height: kind === 'Image' || kind === 'Sprite' ? 256 + i * 32 : null,
    });
  });
}

function fakeUser(tier: MockTier = 'free'): UserProfileDto {
  return {
    id: generateId('user'),
    display_name: 'Mock User',
    email: 'mock@example.com',
    avatar_path: null,
    tier,
    license_key: tier === 'pro' ? 'PRO-MOCK-KEY' : null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

function fakeUsage(): UsageEntry[] {
  return [
    { operation_type: 'generate_image', period: 'month', count: 5, limit: 100, remaining: 95 },
    { operation_type: 'generate_audio', period: 'month', count: 3, limit: 50, remaining: 47 },
    { operation_type: 'generate_code', period: 'month', count: 10, limit: 200, remaining: 190 },
  ];
}

function fakeQuota(): QuotaResult {
  return { allowed: true, remaining: 95, limit: 100, period: 'month' };
}

function fakeProvider(index: number = 0) {
  const providers = [
    { id: 'replicate', name: 'Replicate', kind: 'image_generation', base_url: 'https://api.replicate.com', supported_capabilities: ['image_generation', 'video_generation'], auth_type: 'api_key', enabled: true },
    { id: 'fal', name: 'Fal.ai', kind: 'image_generation', base_url: 'https://api.fal.ai', supported_capabilities: ['image_generation', 'audio_generation'], auth_type: 'api_key', enabled: true },
    { id: 'together', name: 'Together AI', kind: 'text_to_image', base_url: 'https://api.together.ai', supported_capabilities: ['image_generation', 'code_generation'], auth_type: 'api_key', enabled: true },
    { id: 'huggingface', name: 'Hugging Face', kind: 'inference', base_url: 'https://api-inference.huggingface.co', supported_capabilities: ['image_generation', 'text_generation'], auth_type: 'api_key', enabled: true },
  ];
  return providers[index % providers.length];
}

function fakeModelProfile(index: number = 0): ModelProfileDto {
  const profiles = [
    { id: 'profile-1', provider_name: 'replicate', model_id: 'stability-ai/sdxl', display_name: 'SDXL 1.0', capabilities: ['image_generation'], enabled: true, pricing_tier: 'standard', config: {} },
    { id: 'profile-2', provider_name: 'fal', model_id: 'fal-ai/flux/schnell', display_name: 'Flux Schnell', capabilities: ['image_generation'], enabled: true, pricing_tier: 'fast', config: {} },
    { id: 'profile-3', provider_name: 'together', model_id: 'together/llama-3', display_name: 'Llama 3 70B', capabilities: ['code_generation', 'text_generation'], enabled: true, pricing_tier: 'premium', config: {} },
    { id: 'profile-4', provider_name: 'huggingface', model_id: 'stabilityai/stable-diffusion-xl-base-1.0', display_name: 'SDXL Base', capabilities: ['image_generation'], enabled: true, pricing_tier: 'standard', config: {} },
  ];
  return profiles[index % profiles.length];
}

function fakeRoutingRule(index: number = 0): RoutingRuleDto {
  const rules = [
    { id: `rule-${index}`, operation_type: 'generate_image', default_profile_id: 'profile-1', fallback_profile_ids: ['profile-2', 'profile-4'] },
    { id: `rule-${index}`, operation_type: 'generate_audio', default_profile_id: 'profile-2', fallback_profile_ids: ['profile-1'] },
    { id: `rule-${index}`, operation_type: 'generate_code', default_profile_id: 'profile-3', fallback_profile_ids: [] },
    { id: `rule-${index}`, operation_type: 'generate_video', default_profile_id: 'profile-1', fallback_profile_ids: ['profile-2'] },
  ];
  return rules[index % rules.length];
}

function fakePromptTemplate(index: number = 0): PromptTemplateDto {
  const templates = [
    { id: 'tmpl-1', name: 'Game Character', template_text: 'A pixel art game character: {{subject}}, {{style}}', variables: ['subject', 'style'] },
    { id: 'tmpl-2', name: 'Environment Tile', template_text: 'A seamless game environment tile: {{environment}}, {{time_of_day}}', variables: ['environment', 'time_of_day'] },
    { id: 'tmpl-3', name: 'UI Element', template_text: 'A game UI element: {{element_type}}, {{theme}}', variables: ['element_type', 'theme'] },
    { id: 'tmpl-4', name: 'Item/Sprite', template_text: 'A game item sprite: {{item_type}}, {{rarity}}', variables: ['item_type', 'rarity'] },
    { id: 'tmpl-5', name: 'Background', template_text: 'A game background: {{setting}}, {{mood}}', variables: ['setting', 'mood'] },
  ];
  return templates[index % templates.length];
}

function fakeCodeTemplate(engine: string = 'godot'): CodeTemplate {
  return {
    id: generateId('tmpl'),
    name: `${engine} Game Script`,
    description: `A basic ${engine} game script template`,
    engine,
    variables: ['game_title', 'player_name'],
  };
}

// ----------------------------------------------------------------------------
// MockBackend
// ----------------------------------------------------------------------------

export class MockBackend extends MemoryBackend {
  private state: MockState;
  private unlistenFns: Map<string, UnlistenFn> = new Map();

  constructor(state: MockState) {
    super();
    this.state = state;
    this.registerHandlers();
  }

  private recordCall(command: string, args: unknown): void {
    this.state.callHistory.push({ command, args });
  }

  private registerHandlers(): void {
    // Project commands
    this.on('list_projects', (args) => {
      this.recordCall('list_projects', args);
      const includeArchived = (args as { includeArchived?: boolean })?.includeArchived ?? false;
      const projects = this.state.mockData.get('projects') as ProjectResponse[] | undefined;
      if (projects) return projects;
      const mockProjects = [
        fakeProject({ id: 'proj-1', name: 'Dungeon Crawler' }),
        fakeProject({ id: 'proj-2', name: 'Space Shooter' }),
        fakeProject({ id: 'proj-archived', name: 'Archived Project', status: 'archived' }),
      ];
      return includeArchived ? mockProjects : mockProjects.filter(p => p.status !== 'archived');
    });

    this.on('create_project', (args) => {
      this.recordCall('create_project', args);
      const req = (args as { request?: { name?: string; path?: string } })?.request ?? {};
      const newProject = fakeProject({
        id: generateId('proj'),
        name: req.name ?? 'New Project',
        path: req.path ?? '/mock/path',
      });
      const existing = (this.state.mockData.get('projects') as ProjectResponse[] | undefined) ?? [];
      this.state.mockData.set('projects', [...existing, newProject]);
      return newProject;
    });

    this.on('open_project', (args) => {
      this.recordCall('open_project', args);
      const id = (args as { id?: string })?.id ?? '';
      const projects = (this.state.mockData.get('projects') as ProjectResponse[] | undefined) ?? [];
      const project = projects.find(p => p.id === id) ?? fakeProject({ id, name: 'Opened Project' });
      return project;
    });

    this.on('get_project', (args) => {
      this.recordCall('get_project', args);
      const id = (args as { id?: string })?.id ?? '';
      const projects = (this.state.mockData.get('projects') as ProjectResponse[] | undefined) ?? [];
      return projects.find(p => p.id === id) ?? fakeProject({ id, name: 'Project' });
    });

    this.on('rename_project', (args) => {
      this.recordCall('rename_project', args);
      const { id, new_name } = (args as { id?: string; new_name?: string }) ?? {};
      const projects = (this.state.mockData.get('projects') as ProjectResponse[] | undefined) ?? [];
      const project = projects.find(p => p.id === id);
      if (project) project.name = new_name ?? 'Renamed';
      return project ?? fakeProject({ id, name: new_name });
    });

    this.on('archive_project', (_args) => {
      this.recordCall('archive_project', _args);
      return undefined;
    });

    this.on('delete_project', (_args) => {
      this.recordCall('delete_project', _args);
      return undefined;
    });

    // Asset commands
    this.on('list_assets', (args) => {
      this.recordCall('list_assets', args);
      const projectId = (args as { projectId?: string })?.projectId ?? 'proj-mock';
      const assets = this.state.mockData.get('assets') as AssetResponse[] | undefined;
      if (assets) return assets;
      return fakeAssetList(6, projectId);
    });

    this.on('get_asset', (args) => {
      this.recordCall('get_asset', args);
      const id = (args as { id?: string })?.id ?? '';
      return fakeAsset({ id, name: 'Asset' });
    });

    this.on('delete_asset', (_args) => {
      this.recordCall('delete_asset', _args);
      return undefined;
    });

    this.on('import_asset', (args) => {
      this.recordCall('import_asset', args);
      const req = (args as { request?: Partial<AssetResponse> })?.request ?? {};
      return fakeAsset({
        id: generateId('asset'),
        name: req.name ?? 'Imported Asset',
        kind: (req.kind ?? 'Image') as AssetKind,
        project_id: req.project_id ?? 'proj-mock',
      });
    });

    // Job-generating commands - return job_id and simulate lifecycle
    const jobCommands = [
      'generate_image', 'generate_audio', 'synthesize_speech', 'remove_background',
      'convert_pixel_art', 'generate_tile', 'generate_sprite_sheet', 'slice_sprite_sheet',
      'generate_code', 'inpaint_image', 'outpaint_image', 'generate_material',
      'generate_video', 'pack_atlas', 'seamless_texture', 'quick_sprites', 'render_3d_to_sprites',
    ];

    for (const cmd of jobCommands) {
      this.on(cmd, (args) => {
        this.recordCall(cmd, args);
        const jobId = generateId('job');
        if (this.state.mockJobResult === 'success') {
          this.simulateJobSuccess(jobId);
        } else {
          this.simulateJobError(jobId);
        }
        return jobId;
      });
    }

    // Animation commands
    this.on('create_animation', (args) => {
      this.recordCall('create_animation', args);
      return generateId('anim');
    });

    this.on('get_animation', (args) => {
      this.recordCall('get_animation', args);
      const id = (args as { id?: string })?.id ?? '';
      return {
        id,
        project_id: 'proj-mock',
        name: 'Mock Animation',
        kind: 'animation',
        metadata: {
          name: 'Mock Animation',
          frame_asset_ids: [],
          frame_durations_ms: [100],
          loop_animation: true,
          total_duration_ms: 100,
        },
        created_at: new Date().toISOString(),
      };
    });

    this.on('list_animations', (args) => {
      this.recordCall('list_animations', args);
      return [];
    });

    this.on('update_animation', (args) => {
      this.recordCall('update_animation', args);
      return (args as { id?: string })?.id ?? '';
    });

    this.on('delete_animation', (_args) => {
      this.recordCall('delete_animation', _args);
      return undefined;
    });

    this.on('export_animation', (args) => {
      this.recordCall('export_animation', args);
      const jobId = generateId('job');
      this.simulateJobSuccess(jobId);
      return jobId;
    });

    // Collection commands
    this.on('create_collection', (args) => {
      this.recordCall('create_collection', args);
      const req = (args as { request?: { name?: string; project_id?: string } })?.request ?? {};
      return {
        id: generateId('coll'),
        project_id: req.project_id ?? 'proj-mock',
        name: req.name ?? 'New Collection',
        created_at: new Date().toISOString(),
      } as CollectionResponse;
    });

    this.on('list_collections', (args) => {
      this.recordCall('list_collections', args);
      return [];
    });

    this.on('delete_collection', (_args) => {
      this.recordCall('delete_collection', _args);
      return undefined;
    });

    this.on('add_to_collection', (args) => {
      this.recordCall('add_to_collection', args);
      return fakeAsset();
    });

    this.on('remove_from_collection', (args) => {
      this.recordCall('remove_from_collection', args);
      return fakeAsset();
    });

    this.on('get_asset_lineage', (args) => {
      this.recordCall('get_asset_lineage', args);
      return { chain: [] };
    });

    this.on('tag_asset', (args) => {
      this.recordCall('tag_asset', args);
      return fakeAsset();
    });

    this.on('untag_asset', (args) => {
      this.recordCall('untag_asset', args);
      return fakeAsset();
    });

    // Code templates
    this.on('list_code_templates', (args) => {
      this.recordCall('list_code_templates', args);
      const engine = (args as { engine?: string })?.engine ?? 'godot';
      return [fakeCodeTemplate(engine), fakeCodeTemplate(engine)];
    });

    // Identity commands (these use invoke directly, not backend pattern)
    // We handle them here for completeness but they need special wiring
    this.on('get_current_user', (_args) => {
      this.recordCall('get_current_user', _args);
      return fakeUser(this.state.mockTier);
    });

    this.on('update_profile', (args) => {
      this.recordCall('update_profile', args);
      const { displayName, email } = (args as { displayName?: string; email?: string }) ?? {};
      return { ...fakeUser(this.state.mockTier), display_name: displayName ?? 'Mock User' };
    });

    this.on('set_tier', (args) => {
      this.recordCall('set_tier', args);
      const tier = (args as { tier?: string })?.tier as MockTier ?? 'free';
      this.state.mockTier = tier;
      return tier;
    });

    this.on('get_usage', (_args) => {
      this.recordCall('get_usage', _args);
      return fakeUsage();
    });

    this.on('check_quota', (_args) => {
      this.recordCall('check_quota', _args);
      return fakeQuota();
    });

    // Agent commands
    this.on('start_code_agent', (args) => {
      this.recordCall('start_code_agent', args);
      const jobId = generateId('agent-job');
      this.simulateJobSuccess(jobId);
      return jobId;
    });

    // Model config commands
    this.on('list_providers', (_args) => {
      this.recordCall('list_providers', _args);
      return [fakeProvider(0), fakeProvider(1), fakeProvider(2), fakeProvider(3)];
    });

    this.on('get_provider', (args) => {
      this.recordCall('get_provider', args);
      const providerId = (args as { provider_id?: string })?.provider_id ?? 'replicate';
      return fakeProvider(providerId.length % 4);
    });

    this.on('test_provider_connection', (_args) => {
      this.recordCall('test_provider_connection', _args);
      return true;
    });

    this.on('list_model_profiles', (_args) => {
      this.recordCall('list_model_profiles', _args);
      return [fakeModelProfile(0), fakeModelProfile(1), fakeModelProfile(2), fakeModelProfile(3)];
    });

    this.on('create_model_profile', (args) => {
      this.recordCall('create_model_profile', args);
      return fakeModelProfile();
    });

    this.on('update_model_profile', (args) => {
      this.recordCall('update_model_profile', args);
      return fakeModelProfile();
    });

    this.on('delete_model_profile', (_args) => {
      this.recordCall('delete_model_profile', _args);
      return undefined;
    });

    this.on('list_routing_rules', (_args) => {
      this.recordCall('list_routing_rules', _args);
      return [fakeRoutingRule(0), fakeRoutingRule(1), fakeRoutingRule(2), fakeRoutingRule(3)];
    });

    this.on('get_routing_rule', (args) => {
      this.recordCall('get_routing_rule', args);
      const operationType = (args as { operation_type?: string })?.operation_type ?? 'generate_image';
      return fakeRoutingRule(operationType.length % 4);
    });

    this.on('set_routing_rule', (_args) => {
      this.recordCall('set_routing_rule', _args);
      return fakeRoutingRule(0);
    });

    this.on('list_prompt_templates', (_args) => {
      this.recordCall('list_prompt_templates', _args);
      return [fakePromptTemplate(0), fakePromptTemplate(1), fakePromptTemplate(2), fakePromptTemplate(3), fakePromptTemplate(4)];
    });

    this.on('create_prompt_template', (args) => {
      this.recordCall('create_prompt_template', args);
      return fakePromptTemplate();
    });

    this.on('delete_prompt_template', (_args) => {
      this.recordCall('delete_prompt_template', _args);
      return undefined;
    });

    this.on('get_credential_status', (args) => {
      this.recordCall('get_credential_status', args);
      const providerId = (args as { provider_id?: string })?.provider_id ?? 'replicate';
      return { provider_id: providerId, has_credential: true } as CredentialStatusDto;
    });

    this.on('set_credential', (_args) => {
      this.recordCall('set_credential', _args);
      return undefined;
    });

    this.on('delete_credential', (_args) => {
      this.recordCall('delete_credential', _args);
      return undefined;
    });

    this.on('set_provider_enabled', (_args) => {
      this.recordCall('set_provider_enabled', _args);
      return undefined;
    });

    // Publishing commands
    this.on('export_project', (args) => {
      this.recordCall('export_project', args);
      return {
        outputPath: '/mock/export/project',
        fileSizeBytes: 1024 * 1024,
        assetCount: 5,
        manifestPath: '/mock/export/manifest.json',
      };
    });

    this.on('open_itch_io', (_args) => {
      this.recordCall('open_itch_io', _args);
      return undefined;
    });
  }

  private simulateJobSuccess(jobId: string): void {
    // Clear any existing timers for this job
    const existingTimer = this.state.activeJobs.get(jobId);
    if (existingTimer) clearTimeout(existingTimer);

    // Emit progress after 100ms
    const progressTimer = setTimeout(async () => {
      try {
        await emit('job-progress', {
          job_id: jobId,
          progress_percent: 50,
          progress_message: 'Mock generating...',
        });

        // Emit completed after another 100ms
        const completeTimer = setTimeout(async () => {
          try {
            await emit('job-completed', {
              job_id: jobId,
              asset_ids: [generateId('asset')],
            });
          } catch (e) {
            console.warn('[mock] Failed to emit job-completed:', e);
          }
          this.state.activeJobs.delete(jobId);
        }, 100);
        this.state.activeJobs.set(jobId, completeTimer);
      } catch (e) {
        console.warn('[mock] Failed to emit job-progress:', e);
      }
    }, 100);
    this.state.activeJobs.set(jobId, progressTimer);
  }

  private simulateJobError(jobId: string): void {
    const existingTimer = this.state.activeJobs.get(jobId);
    if (existingTimer) clearTimeout(existingTimer);

    const progressTimer = setTimeout(async () => {
      try {
        await emit('job-progress', {
          job_id: jobId,
          progress_percent: 50,
          progress_message: 'Mock error...',
        });

        const errorTimer = setTimeout(async () => {
          try {
            await emit('job-failed', {
              job_id: jobId,
              error_message: 'Mock error: generation failed',
            });
          } catch (e) {
            console.warn('[mock] Failed to emit job-failed:', e);
          }
          this.state.activeJobs.delete(jobId);
        }, 100);
        this.state.activeJobs.set(jobId, errorTimer);
      } catch (e) {
        console.warn('[mock] Failed to emit job-progress:', e);
      }
    }, 100);
    this.state.activeJobs.set(jobId, progressTimer);
  }

  destroy(): void {
    // Clear all active timers
    for (const timer of this.state.activeJobs.values()) {
      clearTimeout(timer);
    }
    this.state.activeJobs.clear();

    // Unlisten all events
    for (const unlisten of this.unlistenFns.values()) {
      unlisten();
    }
    this.unlistenFns.clear();
  }
}

// ----------------------------------------------------------------------------
// Mock Layer Controller
// ----------------------------------------------------------------------------

let mockState: MockState = {
  isMockMode: false,
  mockTier: 'free',
  mockJobResult: 'success',
  mockData: new Map(),
  callHistory: [],
  originalBackends: new Map(),
  activeJobs: new Map(),
  listeners: new Map(),
};

let mockBackend: MockBackend | null = null;

function switchToMockBackend(): void {
  // Save original backends and switch to mock
  mockState.originalBackends.set('projects', projectsApi as unknown as StorageBackend);
  mockState.originalBackends.set('assets', assetsApi as unknown as StorageBackend);
  mockState.originalBackends.set('agent', agentApi as unknown as StorageBackend);
  mockState.originalBackends.set('modelConfig', modelConfigApi as unknown as StorageBackend);

  mockBackend = new MockBackend(mockState);
  projectsApi.setBackend(mockBackend);
  assetsApi.setBackend(mockBackend);
  agentApi.setBackend(mockBackend);
  modelConfigApi.setBackend(mockBackend);
}

function restoreOriginalBackends(): void {
  if (mockBackend) {
    mockBackend.destroy();
    mockBackend = null;
  }

  const projects = mockState.originalBackends.get('projects');
  const assets = mockState.originalBackends.get('assets');
  const agent = mockState.originalBackends.get('agent');
  const modelConfig = mockState.originalBackends.get('modelConfig');

  if (projects) projectsApi.setBackend(projects);
  if (assets) assetsApi.setBackend(assets);
  if (agent) agentApi.setBackend(agent);
  if (modelConfig) modelConfigApi.setBackend(modelConfig);

  mockState.originalBackends.clear();
}

// ----------------------------------------------------------------------------
// Public API (wired into window.__ARTIFEX_DEBUG__)
// ----------------------------------------------------------------------------

export function enableMock(): void {
  if (mockState.isMockMode) return;
  mockState.isMockMode = true;
  mockState.callHistory = [];
  switchToMockBackend();
  console.log('[mock-layer] Mock mode enabled');
}

export function disableMock(): void {
  if (!mockState.isMockMode) return;
  restoreOriginalBackends();
  mockState.isMockMode = false;
  console.log('[mock-layer] Mock mode disabled');
}

export function isMockMode(): boolean {
  return mockState.isMockMode;
}

export function setMockData(key: string, data: unknown): void {
  mockState.mockData.set(key, data);
}

export function setMockTier(tier: MockTier): void {
  mockState.mockTier = tier;
}

export function setMockJobResult(result: MockJobResult): void {
  mockState.mockJobResult = result;
}

export async function simulateJobProgress(jobId: string, percent: number, message: string): Promise<void> {
  await emit('job-progress', { job_id: jobId, progress_percent: percent, progress_message: message });
}

export async function simulateJobCompleted(jobId: string, assetIds: string[]): Promise<void> {
  await emit('job-completed', { job_id: jobId, asset_ids: assetIds });
}

export async function simulateJobFailed(jobId: string, error: string): Promise<void> {
  await emit('job-failed', { job_id: jobId, error_message: error });
}

export function getMockCalls(): Array<{ command: string; args: unknown }> {
  return [...mockState.callHistory];
}

export function getMockCallHistory(command: string): Array<unknown> {
  return mockState.callHistory
    .filter(c => c.command === command)
    .map(c => c.args);
}

export function resetMockCalls(): void {
  mockState.callHistory = [];
}

export function getMockState(): MockState {
  return { ...mockState, mockData: new Map(mockState.mockData), activeJobs: new Map(), originalBackends: new Map() };
}
