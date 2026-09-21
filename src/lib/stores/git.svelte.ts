import { getGitInfo } from '$lib/api/commands';
import type { GitInfo } from '$lib/api/types';

const FOCUS_REFRESH_MIN_MS = 5000;

/**
 * Branch / dirty state for the projects on screen. Purely decorative, so failures are
 * swallowed. Columns report which paths they show; state is re-read when the window
 * regains focus (the user has probably been committing in their editor).
 */
function createGitStore() {
  let info = $state<Record<string, GitInfo>>({});
  const asked = new Set<string>();
  const visibleByColumn = new Map<number, string[]>();
  let lastRefresh = 0;
  let listening = false;

  async function read(paths: string[]) {
    if (paths.length === 0) return;
    try {
      const found = await getGitInfo(paths);
      const next = { ...info };
      for (const p of paths) {
        if (found[p]) next[p] = found[p];
        else delete next[p];
        asked.add(p);
      }
      info = next;
    } catch {
      /* decoration only */
    }
  }

  function refreshVisible() {
    const now = Date.now();
    if (now - lastRefresh < FOCUS_REFRESH_MIN_MS) return;
    lastRefresh = now;
    const paths = new Set<string>();
    for (const list of visibleByColumn.values()) for (const p of list) paths.add(p);
    void read([...paths]);
  }

  function listen() {
    if (listening || typeof window === 'undefined') return;
    listening = true;
    window.addEventListener('focus', refreshVisible);
  }

  return {
    get info() {
      return info;
    },
    /** A column now shows these project paths; read the ones we haven't asked about yet. */
    ensure(columnIndex: number, paths: string[]) {
      listen();
      visibleByColumn.set(columnIndex, paths);
      void read(paths.filter((p) => !asked.has(p)));
    },
    release(columnIndex: number) {
      visibleByColumn.delete(columnIndex);
    },
  };
}

export const gitStore = createGitStore();
