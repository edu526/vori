/**
 * Mock for @tauri-apps/plugin-updater when running in browser (pnpm dev).
 * Always reports "no update" — the real plugin only runs inside Tauri.
 */
export async function check(): Promise<null> {
  return null;
}
