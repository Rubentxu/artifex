import type { StorageBackend } from './storage-backend';

export class MemoryBackend implements StorageBackend {
  private handlers = new Map<string, (...args: unknown[]) => unknown>();

  on(command: string, handler: (...args: unknown[]) => unknown): void {
    this.handlers.set(command, handler);
  }

  async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    const handler = this.handlers.get(command);
    if (!handler) throw new Error(`Unknown command: ${command}`);
    return handler(args) as T;
  }
}
