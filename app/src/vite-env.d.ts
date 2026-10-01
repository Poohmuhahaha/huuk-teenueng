/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Absolute URL of the Rust API (e.g. http://localhost:8787). Unset in
   * production means same-origin `/api` (reverse proxy). */
  readonly VITE_API_URL?: string
  /** "true" forces the in-memory mock even in a production build. */
  readonly VITE_USE_MOCK?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
