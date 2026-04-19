// Pure DOM query helpers for the debug harness — no store imports
import type {
  ElementInfo,
  ViewportInfo,
  TauriContext,
  RouteInfo,
  DOMRectJson,
  NavigationLink,
  DialogInfo,
  ButtonInfo,
  JobStatus,
} from './types';

function rectToJson(rect: DOMRect): DOMRectJson {
  return {
    x: rect.x,
    y: rect.y,
    width: rect.width,
    height: rect.height,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
    left: rect.left,
  };
}

const USEFUL_STYLES = new Set([
  'display',
  'visibility',
  'opacity',
  'width',
  'height',
  'position',
  'overflow',
  'color',
  'backgroundColor',
  'fontSize',
  'fontWeight',
  'border',
  'margin',
  'padding',
]);

function getElementInfo(el: Element): ElementInfo {
  const tag = el.tagName;
  const text = (el.textContent ?? '').trim();
  const classes = Array.from(el.classList);
  const attributes: Record<string, string> = {};
  for (const attr of el.attributes) {
    // Skip event handlers and Svelte internals
    if (!attr.name.startsWith('on') && attr.name !== 'sveltekit:data') {
      attributes[attr.name] = attr.value;
    }
  }
  const computed = window.getComputedStyle(el);
  const styles: Record<string, string> = {};
  for (const prop of USEFUL_STYLES) {
    styles[prop] = computed.getPropertyValue(prop);
  }
  const rect = rectToJson(el.getBoundingClientRect());
  const visible =
    el.offsetParent !== null &&
    computed.display !== 'none' &&
    computed.visibility !== 'hidden' &&
    parseFloat(computed.opacity) !== 0;
  const children = el.children.length;

  return { tag, text, classes, attributes, styles, rect, visible, children };
}

export function getElement(selector: string): ElementInfo | null {
  if (typeof document === 'undefined') return null;
  const el = document.querySelector(selector);
  if (!el) return null;
  return getElementInfo(el);
}

export function getElements(pattern: string): ElementInfo[] {
  if (typeof document === 'undefined') return [];
  const els = document.querySelectorAll(pattern);
  return Array.from(els).map(getElementInfo);
}

export function getViewport(): ViewportInfo {
  return {
    width: window.innerWidth,
    height: window.innerHeight,
    scrollX: window.scrollX,
    scrollY: window.scrollY,
    dpr: window.devicePixelRatio,
    orientation: screen.orientation?.type ?? 'unknown',
  };
}

export function getTauriContext(): TauriContext {
  if (typeof window === 'undefined') {
    return { available: false, platform: null, version: null, arch: null, apis: [] };
  }
  const tauri = (window as unknown as { __TAURI__?: Record<string, unknown> }).__TAURI__;
  if (!tauri) {
    return { available: false, platform: null, version: null, arch: null, apis: [] };
  }
  const tauriObj = tauri as Record<string, unknown>;
  const metadata = tauriObj['metadata'] as Record<string, unknown> | undefined;
  return {
    available: true,
    platform: (metadata?.os as string) ?? null,
    version: (metadata?.version as string) ?? null,
    arch: (metadata?.arch as string) ?? null,
    apis: Object.keys(tauri).filter((k) => !k.startsWith('_')),
  };
}

export function getRoute(): RouteInfo {
  if (typeof window === 'undefined') {
    return { path: '', hash: '', search: {}, origin: '' };
  }
  const { pathname, hash, search, origin } = window.location;
  const parsedSearch: Record<string, string> = {};
  if (search) {
    const params = new URLSearchParams(search);
    params.forEach((value, key) => {
      parsedSearch[key] = value;
    });
  }
  return { path: pathname, hash, search: parsedSearch, origin };
}

export function getNavigation(): NavigationLink[] {
  if (typeof document === 'undefined') return [];
  const navLinks = document.querySelectorAll('nav a');
  const currentPath = window.location.pathname;
  return Array.from(navLinks).map((el) => {
    const href = el.getAttribute('href') ?? '';
    const text = (el.textContent ?? '').trim();
    const active =
      href === currentPath ||
      (href !== '/' && currentPath.startsWith(href));
    return { href, text, active };
  });
}

export function getDialogs(): DialogInfo[] {
  if (typeof document === 'undefined') return [];
  // Look for dialog elements: [role="dialog"], dialog elements, and overlay divs
  const dialogSelectors = [
    '[role="dialog"]',
    'dialog',
    '[class*="dialog-overlay"]',
    '[class*="dialog"]',
  ];
  const results: DialogInfo[] = [];
  for (const selector of dialogSelectors) {
    const els = document.querySelectorAll(selector);
    for (const el of els) {
      const computed = window.getComputedStyle(el);
      const visible =
        computed.display !== 'none' &&
        computed.visibility !== 'hidden' &&
        parseFloat(computed.opacity) !== 0;
      // Get title from common title elements
      const titleEl =
        el.querySelector('[class*="title"]') ||
        el.querySelector('h1, h2, h3') ||
        el.querySelector('[role="heading"]');
      const title = titleEl ? (titleEl.textContent ?? '').trim() : el.tagName;
      const content = (el.textContent ?? '').trim().slice(0, 200);
      results.push({ title, visible, content });
    }
  }
  return results;
}

export function getButtons(textPattern?: string): ButtonInfo[] {
  if (typeof document === 'undefined') return [];
  const buttons = document.querySelectorAll('button');
  return Array.from(buttons)
    .filter((btn) => {
      if (!textPattern) return true;
      const text = (btn.textContent ?? '').trim();
      return text.toLowerCase().includes(textPattern.toLowerCase());
    })
    .map((btn) => {
      const computed = window.getComputedStyle(btn);
      const visible =
        computed.display !== 'none' &&
        computed.visibility !== 'hidden' &&
        parseFloat(computed.opacity) !== 0;
      return {
        text: (btn.textContent ?? '').trim(),
        visible,
        disabled: btn.disabled,
        classes: Array.from(btn.classList),
      };
    });
}

export function getTextContent(selector: string): string {
  if (typeof document === 'undefined') return '';
  const el = document.querySelector(selector);
  if (!el) return '';
  return (el.textContent ?? '').trim();
}

export function hasText(text: string): boolean {
  if (typeof document === 'undefined') return false;
  const body = document.body;
  if (!body) return false;
  return body.textContent?.toLowerCase().includes(text.toLowerCase()) ?? false;
}

export function getActiveJobs(): JobStatus {
  if (typeof document === 'undefined') {
    return { hasActiveJobs: false, statusText: '', progressPercent: null };
  }
  // Look for status bar text that indicates job status
  const statusBar =
    document.querySelector('[class*="status"]') ||
    document.querySelector('footer') ||
    document.querySelector('[class*="StatusBar"]');
  if (!statusBar) {
    return { hasActiveJobs: false, statusText: 'Ready', progressPercent: null };
  }
  const text = (statusBar.textContent ?? '').trim();
  // Check for common job-related patterns
  const processingMatch = text.match(/Processing.*?(\d+)%/);
  const hasActiveJobs =
    text.includes('Processing') ||
    text.includes('Generating') ||
    text.includes('Working');
  let progressPercent: number | null = null;
  if (processingMatch) {
    progressPercent = parseInt(processingMatch[1], 10);
  }
  let statusText = 'Ready';
  if (text.includes('Processing')) statusText = 'Processing';
  else if (text.includes('Generating')) statusText = 'Generating';
  else if (text.includes('Ready')) statusText = 'Ready';
  return { hasActiveJobs, statusText, progressPercent };
}

export function waitForCondition(
  predicateCode: string,
  timeout: number
): boolean {
  if (typeof window === 'undefined') return false;
  const start = Date.now();
  const check = () => {
    try {
      // eslint-disable-next-line no-eval
      const result = eval(predicateCode);
      return result === true;
    } catch {
      return false;
    }
  };
  while (Date.now() - start < timeout) {
    if (check()) return true;
  }
  return false;
}
