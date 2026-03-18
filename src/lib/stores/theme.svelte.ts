function createThemeStore() {
  let os = $state<'macos' | 'windows' | 'linux'>('linux');
  let initialized = $state(false);

  async function init() {
    try {
      const { platform } = await import('@tauri-apps/plugin-os');
      const p = await platform();
      if (p === 'macos') os = 'macos';
      else if (p === 'windows') os = 'windows';
      else os = 'linux';
    } catch {
      os = 'linux';
    }
    document.body.setAttribute('data-os', os);
    initialized = true;
  }

  return {
    get os() {
      return os;
    },
    get initialized() {
      return initialized;
    },
    init,
  };
}

export const themeStore = createThemeStore();
