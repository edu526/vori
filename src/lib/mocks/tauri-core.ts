/**
 * Mock for @tauri-apps/api/core when running in browser (npm run dev).
 * Provides a stateful in-memory implementation of all invoke() commands.
 */
import { mockAppData } from './data';
import type { AppData, Favorites, RecentItem, SearchResult } from '$lib/api/types';

// Mutable state — cloned so tests don't share reference with the exported constant
const state: AppData = structuredClone(mockAppData);

function delay<T>(value: T, ms = 80): Promise<T> {
  return new Promise((resolve) => setTimeout(() => resolve(value), ms));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Args = Record<string, any>;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const handlers: Record<string, (args: Args) => any> = {
  // ── Data loading ───────────────────────────────────────────────────────────
  get_app_data: () => structuredClone(state),

  // ── Categories ────────────────────────────────────────────────────────────
  add_category: ({ key, category }: Args) => {
    state.categories[key] = category;
  },
  update_category: ({ key, category }: Args) => {
    state.categories[key] = { ...state.categories[key], ...category };
  },
  delete_category: ({ key }: Args) => {
    delete state.categories[key];
    for (const [pk, p] of Object.entries(state.projects)) {
      if (p.category === key) delete state.projects[pk];
    }
  },

  // ── Subcategories ─────────────────────────────────────────────────────────
  add_subcategory: ({ categoryKey, key, subcategory }: Args) => {
    if (state.categories[categoryKey]) {
      state.categories[categoryKey].subcategories[key] = subcategory;
    }
  },
  update_subcategory: ({ categoryKey, key, subcategory }: Args) => {
    if (state.categories[categoryKey]?.subcategories[key]) {
      state.categories[categoryKey].subcategories[key] = subcategory;
    }
  },
  delete_subcategory: ({ categoryKey, key }: Args) => {
    if (state.categories[categoryKey]) {
      delete state.categories[categoryKey].subcategories[key];
    }
  },

  // ── Projects ──────────────────────────────────────────────────────────────
  add_project: ({ key, project }: Args) => {
    state.projects[key] = project;
  },
  update_project: ({ key, project }: Args) => {
    state.projects[key] = { ...state.projects[key], ...project };
  },
  delete_project: ({ key }: Args) => {
    delete state.projects[key];
    state.favorites.projects = state.favorites.projects.filter((k) => k !== key);
  },

  // ── Files ─────────────────────────────────────────────────────────────────
  add_file: ({ key, file }: Args) => {
    state.files[key] = file;
  },
  update_file: ({ key, file }: Args) => {
    state.files[key] = { ...state.files[key], ...file };
  },
  delete_file: ({ key }: Args) => {
    delete state.files[key];
    state.favorites.files = state.favorites.files.filter((k) => k !== key);
  },

  // ── Preferences ───────────────────────────────────────────────────────────
  get_preferences: () => structuredClone(state.preferences),
  update_preferences: ({ prefs }: Args) => {
    state.preferences = { ...state.preferences, ...prefs };
  },

  // ── Favorites ─────────────────────────────────────────────────────────────
  get_favorites: () => structuredClone(state.favorites),
  toggle_favorite: ({ key, itemType }: Args): Favorites => {
    const list = itemType === 'project'
      ? state.favorites.projects
      : itemType === 'file'
        ? state.favorites.files
        : state.favorites.categories;
    const idx = list.indexOf(key);
    if (idx >= 0) list.splice(idx, 1);
    else list.push(key);
    return structuredClone(state.favorites);
  },

  // ── Recents ───────────────────────────────────────────────────────────────
  get_recents: () => structuredClone(state.recents),
  add_recent: ({ item }: Args) => {
    state.recents = [item, ...state.recents.filter((r: RecentItem) => r.path !== item.path)].slice(0, 20);
  },

  // ── Launcher ──────────────────────────────────────────────────────────────
  open_project_in_editor: ({ path, editorName }: Args) => {
    console.info(`[mock] open in editor: ${editorName} → ${path}`);
  },
  open_file_in_editor: ({ path }: Args) => {
    console.info(`[mock] open file: ${path}`);
  },
  open_in_terminal: ({ path }: Args) => {
    console.info(`[mock] open terminal at: ${path ?? '~'}`);
  },
  detect_terminals: () => ({ kitty: '/usr/bin/kitty', gnome_terminal: '/usr/bin/gnome-terminal' }),
  detect_editors: () => ({ vscode: '/usr/bin/code', cursor: '/usr/bin/cursor' }),
  get_installed_apps: () => [
    { name: 'Visual Studio Code', exec: '/usr/bin/code' },
    { name: 'Cursor', exec: '/usr/bin/cursor' },
    { name: 'Kitty', exec: '/usr/bin/kitty' },
  ],

  // ── Search ────────────────────────────────────────────────────────────────
  search: ({ query }: Args): SearchResult[] => {
    const q = query.toLowerCase();
    const results: SearchResult[] = [];

    for (const [key, cat] of Object.entries(state.categories)) {
      if (key.includes(q) || cat.description.toLowerCase().includes(q)) {
        results.push({ key, name: key, result_type: 'category' });
      }
      for (const [sk, sub] of Object.entries(cat.subcategories)) {
        if (sk.includes(q) || sub.description.toLowerCase().includes(q)) {
          results.push({ key: sk, name: sk, result_type: 'subcategory', category: key });
        }
      }
    }
    for (const [key, proj] of Object.entries(state.projects)) {
      if (key.includes(q) || proj.path.toLowerCase().includes(q)) {
        results.push({ key, name: key, result_type: 'project', path: proj.path, category: proj.category, subcategory: proj.subcategory });
      }
    }
    for (const [key, file] of Object.entries(state.files)) {
      if (key.includes(q) || file.path.toLowerCase().includes(q)) {
        results.push({ key, name: key, result_type: 'file', path: file.path });
      }
    }
    return results;
  },
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function invoke<T>(command: string, args: Args = {}): Promise<T> {
  const handler = handlers[command];
  if (!handler) {
    console.warn(`[mock] unhandled invoke: ${command}`, args);
    return undefined as T;
  }
  return delay(handler(args) as T);
}
