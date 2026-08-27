import type { InitInput } from './domoxide.js';

export interface DOMPurifyConfig {
  ALLOWED_TAGS?: string[];
  ALLOWED_ATTR?: string[];
  FORBID_TAGS?: string[];
  FORBID_ATTR?: string[];
  ADD_TAGS?: string[];
  ADD_ATTR?: string[];
  USE_PROFILES?: {
    html?: boolean;
    svg?: boolean;
    svgFilters?: boolean;
    mathMl?: boolean;
  };
  [key: string]: any;
}

export interface DOMPurifyInstance {
  /**
   * Sanitizes the untrusted dirty HTML string according to configured options.
   * Matches DOMPurify's synchronous sanitize method.
   */
  sanitize(dirty: string, config?: DOMPurifyConfig): string;
}

export interface CreateDOMPurifyOptions {
  /**
   * Optional custom WebAssembly module, URL, Response, or buffer.
   * In browsers and modern bundlers, this is loaded automatically.
   * In Node.js, pass the loaded `.wasm` buffer or file URL.
   */
  module_or_path?: InitInput | Promise<InitInput>;
}

/**
 * Factory that initializes the DOMOxide WebAssembly engine and returns a
 * DOMPurify-compatible instance.
 */
export function createDOMPurify(options?: CreateDOMPurifyOptions): Promise<DOMPurifyInstance>;

/**
 * Resolves when the engine is initialized; safe to call multiple times.
 */
export function ensureReady(options?: CreateDOMPurifyOptions): Promise<any>;
