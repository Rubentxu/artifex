import { describe, it, expect, beforeEach } from 'vitest';
import { initDebugHarness, destroyDebugHarness } from '../harness';

describe('harness', () => {
  beforeEach(() => {
    // Clean up before each test
    destroyDebugHarness();
  });

  it('initDebugHarness creates window.__ARTIFEX_DEBUG__', () => {
    initDebugHarness();
    expect((globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__).toBeDefined();
  });

  it('window.__ARTIFEX_DEBUG__ has version string', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.version).toBe('string');
    expect(api.version).toBe('1.0.0');
  });

  it('window.__ARTIFEX_DEBUG__ has getStores function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.getStores).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has getStore function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.getStore).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has getRoute function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.getRoute).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has getElement function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.getElement).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has getElements function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.getElements).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has getViewport function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.getViewport).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has getTauriContext function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.getTauriContext).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has snapshot function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.snapshot).toBe('function');
  });

  it('window.__ARTIFEX_DEBUG__ has destroy function', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(typeof api.destroy).toBe('function');
  });

  it('snapshot returns an object with stores, route, viewport, tauri, timestamp', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, Record<string, unknown>>;
    const snap = api.snapshot() as Record<string, unknown>;
    expect(typeof snap.stores).toBe('object');
    expect(typeof snap.route).toBe('object');
    expect(typeof snap.viewport).toBe('object');
    expect(typeof snap.tauri).toBe('object');
    expect(typeof snap.timestamp).toBe('number');
  });

  it('destroy removes __ARTIFEX_DEBUG__', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    api.destroy();
    expect((globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__).toBeUndefined();
  });

  it('destroy is idempotent (calling twice does not throw)', () => {
    initDebugHarness();
    const api = (globalThis as Record<string, unknown>).__ARTIFEX_DEBUG__ as Record<string, unknown>;
    expect(() => {
      api.destroy();
      api.destroy();
    }).not.toThrow();
  });
});
