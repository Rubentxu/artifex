import { describe, it, expect, beforeEach } from 'vitest';
import {
  getElement,
  getElements,
  getViewport,
  getRoute,
  getTauriContext,
  getNavigation,
  getDialogs,
  getButtons,
  getTextContent,
  hasText,
  getActiveJobs,
  waitForCondition,
} from '../dom-inspector';

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

  describe('getNavigation', () => {
    it('returns an array', () => {
      const nav = getNavigation();
      expect(Array.isArray(nav)).toBe(true);
    });

    it('returns navigation links with href, text, active properties', () => {
      const nav = getNavigation();
      for (const link of nav) {
        expect(typeof link.href).toBe('string');
        expect(typeof link.text).toBe('string');
        expect(typeof link.active).toBe('boolean');
      }
    });
  });

  describe('getDialogs', () => {
    it('returns an array', () => {
      const dialogs = getDialogs();
      expect(Array.isArray(dialogs)).toBe(true);
    });

    it('returns dialog info with title, visible, content', () => {
      const dialogs = getDialogs();
      for (const d of dialogs) {
        expect(typeof d.title).toBe('string');
        expect(typeof d.visible).toBe('boolean');
        expect(typeof d.content).toBe('string');
      }
    });
  });

  describe('getButtons', () => {
    it('returns an array', () => {
      const buttons = getButtons();
      expect(Array.isArray(buttons)).toBe(true);
    });

    it('returns button info with text, visible, disabled, classes', () => {
      const buttons = getButtons();
      for (const btn of buttons) {
        expect(typeof btn.text).toBe('string');
        expect(typeof btn.visible).toBe('boolean');
        expect(typeof btn.disabled).toBe('boolean');
        expect(Array.isArray(btn.classes)).toBe(true);
      }
    });

    it('filters by text pattern when provided', () => {
      const allButtons = getButtons();
      const textButtons = getButtons('text');
      expect(textButtons.length).toBeLessThanOrEqual(allButtons.length);
    });
  });

  describe('getTextContent', () => {
    it('returns empty string for non-existent selector', () => {
      const text = getTextContent('#nonexistent');
      expect(text).toBe('');
    });
  });

  describe('hasText', () => {
    it('returns a boolean', () => {
      const result = hasText('artifex');
      expect(typeof result).toBe('boolean');
    });
  });

  describe('getActiveJobs', () => {
    it('returns an object with hasActiveJobs, statusText, progressPercent', () => {
      const status = getActiveJobs();
      expect(typeof status.hasActiveJobs).toBe('boolean');
      expect(typeof status.statusText).toBe('string');
      expect(status.progressPercent === null || typeof status.progressPercent === 'number').toBe(true);
    });
  });

  describe('waitForCondition', () => {
    it('returns boolean', () => {
      const result = waitForCondition('true', 100);
      expect(typeof result).toBe('boolean');
    });

    it('returns true when condition is met', () => {
      const result = waitForCondition('1 + 1 === 2', 100);
      expect(result).toBe(true);
    });

    it('returns false when condition is not met within timeout', () => {
      const result = waitForCondition('false', 100);
      expect(result).toBe(false);
    });
  });
});
