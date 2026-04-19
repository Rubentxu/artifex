// Debug harness TypeScript interfaces for the Artifex E2E testing API

/**
 * Serialized DOMRect — getBoundingClientRect() output
 */
export interface DOMRectJson {
  x: number;
  y: number;
  width: number;
  height: number;
  top: number;
  right: number;
  bottom: number;
  left: number;
}

/**
 * A Svelte store snapshot entry
 */
export interface StoreSnapshot {
  name: string;
  value: unknown;
  type: 'writable' | 'derived';
}

/**
 * Full DOM element information for debug inspection
 */
export interface ElementInfo {
  tag: string;
  text: string;
  classes: string[];
  attributes: Record<string, string>;
  styles: Record<string, string>;
  rect: DOMRectJson;
  visible: boolean;
  children: number;
}

/**
 * Viewport / window dimensions and scroll state
 */
export interface ViewportInfo {
  width: number;
  height: number;
  scrollX: number;
  scrollY: number;
  dpr: number;
  orientation: string;
}

/**
 * Tauri runtime context (availability + platform metadata)
 */
export interface TauriContext {
  available: boolean;
  platform: string | null;
  version: string | null;
  arch: string | null;
  apis: string[];
}

/**
 * Parsed route / URL information
 */
export interface RouteInfo {
  path: string;
  hash: string;
  search: Record<string, string>;
  origin: string;
}

/**
 * Full page snapshot aggregating all debug data
 */
export interface PageSnapshot {
  stores: Record<string, unknown>;
  route: RouteInfo;
  viewport: ViewportInfo;
  tauri: TauriContext;
  timestamp: number;
}

/**
 * The full DebugAPI surface exposed on window.__ARTIFEX_DEBUG__
 */
export interface DebugAPI {
  getStores(): Record<string, unknown>;
  getStore(name: string): unknown;
  getRoute(): RouteInfo;
  getElement(selector: string): ElementInfo | null;
  getElements(pattern: string): ElementInfo[];
  getViewport(): ViewportInfo;
  getTauriContext(): TauriContext;
  snapshot(): PageSnapshot;
  destroy(): void;
  version: string;
}
