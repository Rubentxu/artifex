import { invoke } from '@tauri-apps/api/core';
import type { StorageBackend } from './storage-backend';

export class TauriBackend implements StorageBackend {
  async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(command, args);
  }
}
