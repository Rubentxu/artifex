import type { StorageBackend } from './storage-backend';
import { TauriBackend } from './tauri-backend';
import type { ProjectResponse, CreateProjectRequest } from '$lib/types';

let backend: StorageBackend;
export function setBackend(b: StorageBackend): void {
  backend = b;
}

function getBackend(): StorageBackend {
  if (!backend) backend = new TauriBackend();
  return backend;
}

/**
 * Lists all projects, optionally including archived ones.
 */
export async function listProjects(includeArchived = false): Promise<ProjectResponse[]> {
  return getBackend().invoke<ProjectResponse[]>('list_projects', { includeArchived });
}

/**
 * Creates a new project.
 */
export async function createProject(name: string, path: string): Promise<ProjectResponse> {
  const request: CreateProjectRequest = { name, path };
  return getBackend().invoke<ProjectResponse>('create_project', { request });
}

/**
 * Opens a project, setting it as the current active project.
 */
export async function openProject(id: string): Promise<ProjectResponse> {
  return getBackend().invoke<ProjectResponse>('open_project', { id });
}

/**
 * Gets a single project by ID.
 */
export async function getProject(id: string): Promise<ProjectResponse> {
  return getBackend().invoke<ProjectResponse>('get_project', { id });
}

/**
 * Renames a project.
 */
export async function renameProject(id: string, newName: string): Promise<ProjectResponse> {
  return getBackend().invoke<ProjectResponse>('rename_project', { id, new_name: newName });
}

/**
 * Archives a project.
 */
export async function archiveProject(id: string): Promise<void> {
  return getBackend().invoke<void>('archive_project', { id });
}

/**
 * Deletes a project (hard delete - not implemented in Phase 0).
 */
export async function deleteProject(id: string): Promise<void> {
  return getBackend().invoke<void>('delete_project', { id });
}
