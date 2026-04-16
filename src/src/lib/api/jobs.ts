import type { StorageBackend } from './storage-backend';
import { TauriBackend } from './tauri-backend';
import type { JobResponse } from '$lib/types';

let backend: StorageBackend;
export function setBackend(b: StorageBackend): void {
  backend = b;
}

function getBackend(): StorageBackend {
  if (!backend) backend = new TauriBackend();
  return backend;
}

export async function listJobs(projectId: string, jobType?: string): Promise<JobResponse[]> {
  return getBackend().invoke<JobResponse[]>('list_jobs', { projectId, jobType: jobType || null });
}

export async function getJob(id: string): Promise<JobResponse> {
  return getBackend().invoke<JobResponse>('get_job', { id });
}

export async function cancelJob(id: string): Promise<void> {
  return getBackend().invoke<void>('cancel_job', { id });
}