import { describe, it, expect, beforeEach } from 'vitest';
import { initStoreInspector, getStores, getStore, destroyStoreInspector } from '../store-inspector';

describe('store-inspector', () => {
  beforeEach(() => {
    // Always start fresh
    destroyStoreInspector();
  });

  it('initStoreInspector does not throw', () => {
    expect(() => initStoreInspector()).not.toThrow();
  });

  it('getStores returns an object with expected store key prefixes', () => {
    initStoreInspector();
    const stores = getStores();
    expect(typeof stores).toBe('object');
    // Should have keys for ui, project, asset, identity, agent
    const keys = Object.keys(stores);
    expect(keys.some(k => k.startsWith('ui.'))).toBe(true);
    expect(keys.some(k => k === 'project' || k.startsWith('project.'))).toBe(true);
    expect(keys.some(k => k === 'asset' || k.startsWith('asset.'))).toBe(true);
    expect(keys.some(k => k === 'identity' || k.startsWith('identity.'))).toBe(true);
    expect(keys.some(k => k === 'agent' || k.startsWith('agent.'))).toBe(true);
  });

  it('getStore returns undefined for unknown store name', () => {
    initStoreInspector();
    expect(getStore('nonexistent.store')).toBeUndefined();
  });

  it('getStore returns a value for a known store name', () => {
    initStoreInspector();
    // 'ui.sidebarCollapsed' should exist per store registry
    const value = getStore('ui.sidebarCollapsed');
    expect(value).not.toBeUndefined();
  });

  it('destroyStoreInspector cleans up internal state', () => {
    initStoreInspector();
    destroyStoreInspector();
    // After destroy, getStores should return {} (empty, since subscriptions cleared)
    const stores = getStores();
    expect(Object.keys(stores).length).toBe(0);
  });
});
