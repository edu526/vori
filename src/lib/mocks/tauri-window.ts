/**
 * Mock for @tauri-apps/api/window when running in browser (npm run dev).
 */
export function getCurrentWindow() {
  return {
    close: () => {
      console.info('[mock] window.close() called');
    },
  };
}
