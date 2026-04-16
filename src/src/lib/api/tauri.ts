import { invoke } from '@tauri-apps/api/core';

/**
 * Generic wrapper for Tauri IPC invoke commands.
 * Handles automatic conversion between camelCase (TypeScript) and snake_case (Rust).
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  return invoke<T>(command, args);
}
