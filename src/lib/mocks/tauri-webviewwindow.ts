/**
 * Mock for @tauri-apps/api/webviewWindow when running in browser (npm run dev).
 * setZoom applies CSS font-size scaling so the dev experience mirrors Tauri.
 */
export function getCurrentWebviewWindow() {
  return {
    setZoom: async (factor: number) => {
      document.documentElement.style.fontSize = `${14 * factor}px`;
    },
  };
}
