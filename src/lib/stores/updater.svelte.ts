import { check, type Update } from '@tauri-apps/plugin-updater';
import { ask } from '@tauri-apps/plugin-dialog';

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
      // "Could not fetch a valid release JSON" = endpoint returned 404/no release
      // yet. Not an error from the user's perspective — they're just up to date.
      if (msg.includes('Could not fetch a valid release JSON')) {
        state = 'up-to-date';
        available = null;
        return;
      }
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
    await available.install();
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
