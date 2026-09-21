/**
 * Mock for @tauri-apps/api/event when running in browser (pnpm dev): there is no backend
 * to emit anything, so listeners simply never fire.
 */
export type UnlistenFn = () => void;

export async function listen(_event: string, _handler: (e: unknown) => void): Promise<UnlistenFn> {
  return () => {};
}
