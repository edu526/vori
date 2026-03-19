/**
 * Mock for @tauri-apps/api/window when running in browser (npm run dev).
 */
export function getCurrentWindow() {
  return {
    close: () => console.info('[mock] window.close()'),
    minimize: () => console.info('[mock] window.minimize()'),
    toggleMaximize: () => console.info('[mock] window.toggleMaximize()'),
    isMaximized: async () => false,
    startDragging: () => console.info('[mock] window.startDragging()'),
  };
}
