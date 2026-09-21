/**
 * Mock for @tauri-apps/plugin-os when running in browser (pnpm dev).
 * Reports Linux so the Linux-only bits of the UI can be looked at.
 */
export function type(): string {
  return 'linux';
}
