import type { StorageBackend } from './storage-backend';
import { TauriBackend } from './tauri-backend';
import type { CodeAgentRequest } from '$lib/types';

let backend: StorageBackend;
export function setBackend(b: StorageBackend): void {
  backend = b;
}

function getBackend(): StorageBackend {
  if (!backend) backend = new TauriBackend();
  return backend;
}

export async function startCodeAgent(request: CodeAgentRequest): Promise<string> {
  return getBackend().invoke<string>('start_code_agent', {
    projectId: request.projectId,
    engine: request.engine,
    prompt: request.prompt,
    modelId: request.modelId ?? null,
    maxDurationSecs: request.maxDurationSecs ?? null,
  });
}
