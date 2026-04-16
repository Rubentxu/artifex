// TypeScript types matching Rust DTOs (camelCase for JSON)
export interface ProjectResponse {
  id: string;
  name: string;
  path: string;
  status: ProjectStatus;
  created_at: string;
  updated_at: string;
}

export type ProjectStatus = 'active' | 'archived';

export interface CreateProjectRequest {
  name: string;
  path: string;
}

export interface JobResponse {
  id: string;
  project_id: string;
  job_type: string;
  status: string;
  operation: Record<string, unknown>;
  progress_percent: number;
  progress_message: string | null;
  error_message: string | null;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}
