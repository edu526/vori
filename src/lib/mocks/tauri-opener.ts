/**
 * Mock for @tauri-apps/plugin-opener when running in browser (pnpm dev).
 */
export async function revealItemInDir(path: string | string[]): Promise<void> {
  console.info(`[mock] reveal in file manager: ${path}`);
}
