/**
 * Mock for @tauri-apps/plugin-process when running in browser (pnpm dev).
 */
export async function relaunch(): Promise<void> {
  location.reload();
}
