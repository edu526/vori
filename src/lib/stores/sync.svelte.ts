import { scanFolder } from '$lib/api/commands';
import { configStore } from '$lib/stores/config.svelte';

const IGNORED_KEY = 'vori.ignoredNewFolders';
const FOCUS_CHECK_MIN_MS = 2 * 60 * 1000;

function loadIgnored(): Set<string> {
  try {
    const raw = localStorage.getItem(IGNORED_KEY);
    return new Set(raw ? (JSON.parse(raw) as string[]) : []);
  } catch {
    return new Set();
  }
}

function saveIgnored(set: Set<string>) {
  try {
    localStorage.setItem(IGNORED_KEY, JSON.stringify([...set]));
  } catch {
    /* dismissing is a convenience; worst case the badge comes back */
  }
}

/**
 * Categories bound to a folder (`source_path`) can gain projects outside Vori. This
 * scans those folders in the background and reports which project folders are not
 * imported yet, so the category can show a "+N" badge and offer to import them.
 */
function createSyncStore() {
  // Project folders found on disk per category, from the last scan.
  let scanned = $state<Record<string, string[]>>({});
  let ignored = $state<Set<string>>(loadIgnored());
  let checking = false;
  let lastCheck = 0;
  let started = false;

  const known = $derived(new Set(Object.values(configStore.projects).map((p) => p.path)));

  function newPaths(categoryKey: string): string[] {
    return (scanned[categoryKey] ?? []).filter((p) => !known.has(p) && !ignored.has(p));
  }

  async function check() {
    if (checking) return;
    checking = true;
    lastCheck = Date.now();
    try {
      const next: Record<string, string[]> = {};
      for (const [key, cat] of Object.entries(configStore.categories)) {
        if (!cat.source_path) continue;
        try {
          next[key] = (await scanFolder(cat.source_path)).map((p) => p.path);
        } catch {
          /* folder gone or unreadable: nothing to report for it */
        }
      }
      scanned = next;
    } finally {
      checking = false;
    }
  }

  function start() {
    if (started || typeof window === 'undefined') return;
    started = true;
    setTimeout(check, 1500); // let the first paint finish before touching the disk
    window.addEventListener('focus', () => {
      if (Date.now() - lastCheck > FOCUS_CHECK_MIN_MS) void check();
    });
  }

  /** Stop suggesting the folders currently pending for this category. */
  function dismiss(categoryKey: string) {
    const next = new Set(ignored);
    for (const p of newPaths(categoryKey)) next.add(p);
    ignored = next;
    saveIgnored(next);
  }

  return {
    newPaths,
    newCount: (categoryKey: string) => newPaths(categoryKey).length,
    check,
    start,
    dismiss,
  };
}

export const syncStore = createSyncStore();
