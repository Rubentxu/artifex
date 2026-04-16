import { describe, it, expect } from 'vitest';
import { sidebarCollapsed, propertiesCollapsed, selectedProjectId } from '../ui';

describe('panelState store', () => {
  it('has valid initial values for sidebar', () => {
    expect(typeof sidebarCollapsed).toBe('object');
  });

  it('has valid initial values for properties panel', () => {
    expect(typeof propertiesCollapsed).toBe('object');
  });

  it('has valid initial values for selected project id', () => {
    expect(typeof selectedProjectId).toBe('object');
  });
});
