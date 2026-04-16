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

export interface ListJobsFilters {
  jobType?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

export async function listJobs(projectId: string, filters?: ListJobsFilters): Promise<JobResponse[]> {
  return getBackend().invoke<JobResponse[]>('list_jobs', {
    projectId,
    jobType: filters?.jobType ?? null,
    status: filters?.status ?? null,
    limit: filters?.limit ?? null,
    offset: filters?.offset ?? null,
  });
}

export async function getJob(id: string): Promise<JobResponse> {
  return getBackend().invoke<JobResponse>('get_job', { id });
}

export async function cancelJob(id: string): Promise<void> {
  return getBackend().invoke<void>('cancel_job', { id });
}