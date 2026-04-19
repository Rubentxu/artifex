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
 * Navigation link information
 */
export interface NavigationLink {
  href: string;
  text: string;
  active: boolean;
}

/**
 * Dialog information
 */
export interface DialogInfo {
  title: string;
  visible: boolean;
  content: string;
}

/**
 * Button information
 */
export interface ButtonInfo {
  text: string;
  visible: boolean;
  disabled: boolean;
  classes: string[];
}

/**
 * Job status from StatusBar
 */
export interface JobStatus {
  hasActiveJobs: boolean;
  statusText: string;
  progressPercent: number | null;
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
 * The MockAPI surface for controlling the mock layer
 */
export interface MockAPI {
  // Enable/disable mock mode
  enableMock(): void;
  disableMock(): void;
  isMockMode(): boolean;
  // Configure mock data
  setMockData(key: string, data: unknown): void;
  setMockTier(tier: 'free' | 'pro'): void;
  setMockJobResult(result: 'success' | 'error'): void;
  // Simulate job lifecycle manually
  simulateJobProgress(jobId: string, percent: number, message: string): Promise<void>;
  simulateJobCompleted(jobId: string, assetIds: string[]): Promise<void>;
  simulateJobFailed(jobId: string, error: string): Promise<void>;
  // Get mock state
  getMockCalls(): Array<{ command: string; args: unknown }>;
  getMockCallHistory(command: string): Array<unknown>;
  resetMockCalls(): void;
  getMockState(): unknown;
}

/**
 * The full DebugAPI surface exposed on window.__ARTIFEX_DEBUG__
 */
export interface DebugAPI {
  // Store inspection
  getStores(): Record<string, unknown>;
  getStore(name: string): unknown;
  // Route/URL
  getRoute(): RouteInfo;
  // DOM queries
  getElement(selector: string): ElementInfo | null;
  getElements(pattern: string): ElementInfo[];
  getNavigation(): NavigationLink[];
  getDialogs(): DialogInfo[];
  getButtons(textPattern?: string): ButtonInfo[];
  getTextContent(selector: string): string;
  hasText(text: string): boolean;
  // Viewport
  getViewport(): ViewportInfo;
  // StatusBar
  getActiveJobs(): JobStatus;
  // Tauri
  getTauriContext(): TauriContext;
  // Utilities
  waitForCondition(predicateCode: string, timeout: number): boolean;
  snapshot(): PageSnapshot;
  destroy(): void;
  version: string;
  // Mock layer (only active in DEV)
  mock?: MockAPI;
}
