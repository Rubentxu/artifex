import type { StorageBackend } from './storage-backend';
import { TauriBackend } from './tauri-backend';
import type { ProjectResponse, CreateProjectRequest } from '$lib/types';

// Types for model config
export interface ProviderDto {
  id: string;
  name: string;
  kind: string;
  base_url: string;
  supported_capabilities: string[];
  auth_type: string;
  enabled: boolean;
}

export interface ModelProfileDto {
  id: string;
  provider_name: string;
  model_id: string;
  display_name: string;
  capabilities: string[];
  enabled: boolean;
  pricing_tier: string;
  config: Record<string, unknown>;
}

export interface RoutingRuleDto {
  id: string;
  operation_type: string;
  default_profile_id: string;
  fallback_profile_ids: string[];
}

export interface PromptTemplateDto {
  id: string;
  name: string;
  template_text: string;
  variables: string[];
}

export interface CredentialStatusDto {
  provider_id: string;
  has_credential: boolean;
}

let backend: StorageBackend;
export function setBackend(b: StorageBackend): void {
  backend = b;
}

function getBackend(): StorageBackend {
  if (!backend) backend = new TauriBackend();
  return backend;
}

// Provider management
export async function listProviders(): Promise<ProviderDto[]> {
  return getBackend().invoke<ProviderDto[]>('list_providers');
}

export async function getProvider(providerId: string): Promise<ProviderDto | null> {
  return getBackend().invoke<ProviderDto | null>('get_provider', { provider_id: providerId });
}

export async function testProviderConnection(providerId: string): Promise<boolean> {
  return getBackend().invoke<boolean>('test_provider_connection', { provider_id: providerId });
}

// Model profiles
export async function listModelProfiles(): Promise<ModelProfileDto[]> {
  return getBackend().invoke<ModelProfileDto[]>('list_model_profiles');
}

export async function createModelProfile(
  providerName: string,
  modelId: string,
  displayName: string,
  capabilities: string[]
): Promise<ModelProfileDto> {
  return getBackend().invoke<ModelProfileDto>('create_model_profile', {
    provider_name: providerName,
    model_id: modelId,
    display_name: displayName,
    capabilities,
  });
}

export async function updateModelProfile(
  id: string,
  providerName: string,
  modelId: string,
  displayName: string,
  capabilities: string[],
  enabled: boolean,
  pricingTier: string,
  config: Record<string, unknown>
): Promise<ModelProfileDto> {
  return getBackend().invoke<ModelProfileDto>('update_model_profile', {
    id,
    provider_name: providerName,
    model_id: modelId,
    display_name: displayName,
    capabilities,
    enabled,
    pricing_tier: pricingTier,
    config,
  });
}

export async function deleteModelProfile(id: string): Promise<void> {
  return getBackend().invoke<void>('delete_model_profile', { id });
}

// Routing rules
export async function listRoutingRules(): Promise<RoutingRuleDto[]> {
  return getBackend().invoke<RoutingRuleDto[]>('list_routing_rules');
}

export async function getRoutingRule(operationType: string): Promise<RoutingRuleDto | null> {
  const rules = await listRoutingRules();
  return rules.find(r => r.operation_type === operationType) ?? null;
}

export async function setRoutingRule(
  operationType: string,
  defaultProfileId: string,
  fallbackProfileIds: string[]
): Promise<RoutingRuleDto> {
  return getBackend().invoke<RoutingRuleDto>('set_routing_rule', {
    operation_type: operationType,
    default_profile_id: defaultProfileId,
    fallback_profile_ids: fallbackProfileIds,
  });
}

// Prompt templates
export async function listPromptTemplates(): Promise<PromptTemplateDto[]> {
  return getBackend().invoke<PromptTemplateDto[]>('list_prompt_templates');
}

export async function createPromptTemplate(
  name: string,
  templateText: string
): Promise<PromptTemplateDto> {
  return getBackend().invoke<PromptTemplateDto>('create_prompt_template', {
    name,
    template_text: templateText,
  });
}

export async function deletePromptTemplate(id: string): Promise<void> {
  return getBackend().invoke<void>('delete_prompt_template', { id });
}

// Credentials
export async function getCredentialStatus(providerId: string): Promise<CredentialStatusDto> {
  return getBackend().invoke<CredentialStatusDto>('get_credential_status', { provider_id: providerId });
}

export async function setCredential(providerId: string, apiKey: string): Promise<void> {
  return getBackend().invoke<void>('set_credential', { provider_id: providerId, api_key: apiKey });
}

export async function deleteCredential(providerId: string): Promise<void> {
  return getBackend().invoke<void>('delete_credential', { provider_id: providerId });
}

export async function setProviderEnabled(providerId: string, enabled: boolean): Promise<void> {
  return getBackend().invoke<void>('set_provider_enabled', { provider_id: providerId, enabled });
}
