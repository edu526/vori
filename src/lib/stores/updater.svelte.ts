import { check, type Update } from '@tauri-apps/plugin-updater';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdaterState = 'idle' | 'checking' | 'up-to-date' | 'available' | 'error';

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body?: string;
  install: () => Promise<void>;
}

function createUpdaterStore() {
  let state = $state<UpdaterState>('idle');
  let available = $state<UpdateInfo | null>(null);
  let lastError = $state('');

  async function refresh(): Promise<void> {
    if (state === 'checking') return;
    state = 'checking';
    lastError = '';
    try {
      const update = await check();
      if (!update) {
        state = 'up-to-date';
        available = null;
        return;
      }
      available = {
        version: update.version,
        currentVersion: update.currentVersion,
        body: update.body,
        install: () => update.downloadAndInstall(),
      };
      state = 'available';
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      state = 'error';
      lastError = msg;
    }
  }

  async function promptInstall(): Promise<boolean> {
    if (!available) return false;
    const yes = await ask(
      `Update to v${available.version}?\n\nCurrent: v${available.currentVersion}`,
      { title: 'Update available', kind: 'info', okLabel: 'Install', cancelLabel: 'Later' },
    );
    if (!yes) return false;
    try {
      await available.install();
    } catch (e) {
      lastError = e instanceof Error ? e.message : String(e);
      state = 'error';
      await message(`Could not install the update.

${lastError}`, {
        title: 'Update failed',
        kind: 'error',
      });
      return false;
    }
    // Windows' installer closes the app on its own; macOS/Linux need an explicit restart.
    await relaunch();
    return true;
  }

  return {
    get state() {
      return state;
    },
    get available() {
      return available;
    },
    get lastError() {
      return lastError;
    },
    refresh,
    promptInstall,
  };
}

export const updaterStore = createUpdaterStore();
