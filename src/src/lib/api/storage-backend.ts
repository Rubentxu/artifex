/**
 * Runtime-agnostic storage backend interface for Tauri IPC.
 * Allows injection of in-memory implementation for testing.
 */
export interface StorageBackend {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}
