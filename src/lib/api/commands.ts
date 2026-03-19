import { invoke } from '@tauri-apps/api/core';

import type {
  AppData,
  Category,
  Favorites,
  FileEntry,
  Preferences,
  Project,
  RecentItem,
  SearchResult,
  Subcategory,
} from './types';

// ── Data loading ──────────────────────────────────────────────────────────────

export const getAppData = () => invoke<AppData>('get_app_data');

// ── Categories ────────────────────────────────────────────────────────────────

export const addCategory = (key: string, category: Category) =>
  invoke<void>('add_category', { key, category });

export const updateCategory = (key: string, category: Category) =>
  invoke<void>('update_category', { key, category });

export const deleteCategory = (key: string) => invoke<void>('delete_category', { key });

export const addSubcategory = (categoryKey: string, key: string, subcategory: Subcategory) =>
  invoke<void>('add_subcategory', { categoryKey, key, subcategory });

export const updateSubcategory = (categoryKey: string, key: string, subcategory: Subcategory) =>
  invoke<void>('update_subcategory', { categoryKey, key, subcategory });

export const deleteSubcategory = (categoryKey: string, key: string) =>
  invoke<void>('delete_subcategory', { categoryKey, key });

// ── Projects ──────────────────────────────────────────────────────────────────

export const addProject = (key: string, project: Project) =>
  invoke<void>('add_project', { key, project });

export const updateProject = (key: string, project: Project) =>
  invoke<void>('update_project', { key, project });

export const deleteProject = (key: string) => invoke<void>('delete_project', { key });

// ── Files ─────────────────────────────────────────────────────────────────────

export const addFile = (key: string, file: FileEntry) =>
  invoke<void>('add_file', { key, file });

export const updateFile = (key: string, file: FileEntry) =>
  invoke<void>('update_file', { key, file });

export const deleteFile = (key: string) => invoke<void>('delete_file', { key });

// ── Preferences ───────────────────────────────────────────────────────────────

export const getPreferences = () => invoke<Preferences>('get_preferences');

export const updatePreferences = (prefs: Preferences) =>
  invoke<void>('update_preferences', { prefs });

// ── Favorites ─────────────────────────────────────────────────────────────────

export const getFavorites = () => invoke<Favorites>('get_favorites');

export const toggleFavorite = (key: string, itemType: 'project' | 'file' | 'category') =>
  invoke<Favorites>('toggle_favorite', { key, itemType });

// ── Recents ───────────────────────────────────────────────────────────────────

export const getRecents = () => invoke<RecentItem[]>('get_recents');

export const addRecent = (item: RecentItem) => invoke<void>('add_recent', { item });

// ── Launcher ──────────────────────────────────────────────────────────────────

export const openProjectInEditor = (path: string, editorName: string) =>
  invoke<void>('open_project_in_editor', { path, editorName });

export const openFileInEditor = (path: string, textEditor?: string) =>
  invoke<void>('open_file_in_editor', { path, textEditor });

export const openInTerminal = (path?: string) =>
  invoke<void>('open_in_terminal', { path });

export const detectTerminals = () =>
  invoke<Record<string, string>>('detect_terminals');

export const detectEditors = () =>
  invoke<Record<string, string>>('detect_editors');

export const getInstalledApps = () =>
  invoke<{ name: string; exec: string }[]>('get_installed_apps');

// ── Search ────────────────────────────────────────────────────────────────────

export const search = (query: string) => invoke<SearchResult[]>('search', { query });
