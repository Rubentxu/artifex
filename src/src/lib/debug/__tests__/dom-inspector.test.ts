import { describe, it, expect } from 'vitest';
import { getElement, getElements, getViewport, getRoute, getTauriContext } from '../dom-inspector';

describe('dom-inspector', () => {
  describe('getElement', () => {
    it('returns null for non-existent selector', () => {
      const result = getElement('#nonexistent-element-xyz');
      expect(result).toBeNull();
    });
  });

  describe('getViewport', () => {
    it('returns valid numeric dimensions', () => {
      const viewport = getViewport();
      expect(typeof viewport.width).toBe('number');
      expect(typeof viewport.height).toBe('number');
      expect(viewport.width).toBeGreaterThan(0);
      expect(viewport.height).toBeGreaterThan(0);
    });

    it('returns scrollX and scrollY as numbers', () => {
      const viewport = getViewport();
      expect(typeof viewport.scrollX).toBe('number');
      expect(typeof viewport.scrollY).toBe('number');
    });

    it('returns dpr as a number', () => {
      const viewport = getViewport();
      expect(typeof viewport.dpr).toBe('number');
      expect(viewport.dpr).toBeGreaterThan(0);
    });
  });

  describe('getRoute', () => {
    it('returns an object with path property', () => {
      const route = getRoute();
      expect(typeof route).toBe('object');
      expect(typeof route.path).toBe('string');
    });

    it('returns search as a Record', () => {
      const route = getRoute();
      expect(typeof route.search).toBe('object');
    });

    it('returns origin as a string', () => {
      const route = getRoute();
      expect(typeof route.origin).toBe('string');
    });
  });

  describe('getTauriContext', () => {
    it('returns an object with available boolean', () => {
      const ctx = getTauriContext();
      expect(typeof ctx).toBe('object');
      expect(typeof ctx.available).toBe('boolean');
    });
  });
});
