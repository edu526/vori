import { check, type Update } from '@tauri-apps/plugin-updater';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { notesToPlainText } from '$lib/utils/releaseNotes';

export type UpdaterState =
  | 'idle'
  | 'checking'
  | 'up-to-date'
  | 'available'
  | 'downloading'
  | 'error';

const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000;

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body?: string;
  install: (onProgress: (percent: number | null) => void) => Promise<void>;
}

function createUpdaterStore() {
  let state = $state<UpdaterState>('idle');
  let available = $state<UpdateInfo | null>(null);
  let lastError = $state('');
  // 0-100 while downloading; null when the size is unknown.
  let progress = $state<number | null>(null);

  async function refresh(): Promise<void> {
    if (state === 'checking' || state === 'downloading') return;
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
        install: (onProgress) => {
          let total = 0;
          let received = 0;
          return update.downloadAndInstall((event) => {
            if (event.event === 'Started') {
              total = event.data.contentLength ?? 0;
              onProgress(total ? 0 : null);
            } else if (event.event === 'Progress') {
              received += event.data.chunkLength;
              onProgress(total ? Math.min(100, Math.round((received / total) * 100)) : null);
            }
          });
        },
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
    const notes = notesToPlainText(available.body);
    const yes = await ask(
      `Update to v${available.version}?\n\nCurrent: v${available.currentVersion}` +
        (notes ? `\n\nWhat's new:\n${notes}` : ''),
      { title: 'Update available', kind: 'info', okLabel: 'Install', cancelLabel: 'Later' },
    );
    if (!yes) return false;
    state = 'downloading';
    progress = null;
    try {
      await available.install((percent) => {
        progress = percent;
      });
    } catch (e) {
      lastError = e instanceof Error ? e.message : String(e);
      state = 'error';
      progress = null;
      await message(`Could not install the update.\n\n${lastError}`, {
        title: 'Update failed',
        kind: 'error',
      });
      return false;
    }
    // Windows' installer closes the app on its own; macOS/Linux need an explicit restart.
    await relaunch();
    return true;
  }

  /** Silent check now and then once a day. Manual checks go through the version chip. */
  function startAutoCheck(): () => void {
    void refresh();
    const timer = setInterval(() => {
      if (state !== 'available' && state !== 'downloading') void refresh();
    }, CHECK_INTERVAL_MS);
    return () => clearInterval(timer);
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
    get progress() {
      return progress;
    },
    refresh,
    startAutoCheck,
    promptInstall,
  };
}

export const updaterStore = createUpdaterStore();
