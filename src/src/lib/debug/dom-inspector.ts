// Pure DOM query helpers for the debug harness — no store imports
import type {
  ElementInfo,
  ViewportInfo,
  TauriContext,
  RouteInfo,
  DOMRectJson,
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
