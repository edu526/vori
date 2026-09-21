import { invoke } from '@tauri-apps/api/core';

import type {
  AppData,
  DetectedWorkspace,
  Favorites,
  FileEntry,
  CliRequest,
  GitInfo,
  Preferences,
  Project,
  RecentItem,
  ScannedProject,
  SearchResult,
} from './types';

// ── Data loading ──────────────────────────────────────────────────────────────

export const getAppData = () => invoke<AppData>('get_app_data');

// ── Categories ────────────────────────────────────────────────────────────────

export const addCategory = (key: string, parent: string | null, sourcePath?: string | null) =>
  invoke<void>('add_category', { key, parent, sourcePath });

export const updateCategory = (key: string, parent: string | null, sourcePath?: string | null) =>
  invoke<void>('update_category', { key, parent, sourcePath });

export const deleteCategory = (key: string) => invoke<void>('delete_category', { key });

// ── Claude profiles ───────────────────────────────────────────────────────────

/** Assign a Claude profile to a category or project; `null` clears it (inherit). */
export const setClaudeProfile = (
  target: 'category' | 'project',
  key: string,
  profile: string | null,
) => invoke<void>('set_claude_profile', { target, key, profile });

// ── Projects ──────────────────────────────────────────────────────────────────

export const addProject = (key: string, project: Project) =>
  invoke<void>('add_project', { key, project });

export const updateProject = (key: string, project: Project) =>
  invoke<void>('update_project', { key, project });

export const deleteProject = (key: string) => invoke<void>('delete_project', { key });

export const bulkImportProjects = (entries: [string, Project][]) =>
  invoke<void>('bulk_import_projects', { entries });

// ── Files ─────────────────────────────────────────────────────────────────────

export const addFile = (key: string, file: FileEntry) =>
  invoke<void>('add_file', { key, file });

export const updateFile = (key: string, file: FileEntry) =>
  invoke<void>('update_file', { key, file });

export const deleteFile = (key: string) => invoke<void>('delete_file', { key });

// ── Files — read/write text content ───────────────────────────────────────────

export const readTextFile = (path: string) =>
  invoke<string>('read_text_file', { path });

export const writeTextFile = (path: string, content: string) =>
  invoke<void>('write_text_file', { path, content });

export const writeTextFileElevated = (path: string, content: string) =>
  invoke<void>('write_text_file_elevated', { path, content });

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

// ── Workspace selection ────────────────────────────────────────────────────────

export interface WorkspaceSelectionEntry {
  path: string;
  label: string;
}

export const getWorkspaceSelection = () =>
  invoke<WorkspaceSelectionEntry[]>('get_workspace_selection');

export const setWorkspaceSelection = (entries: WorkspaceSelectionEntry[]) =>
  invoke<void>('set_workspace_selection', { entries });

// ── Launcher ──────────────────────────────────────────────────────────────────

export const openProjectInEditor = async (path: string, editorName: string) => {
  console.info('[vori][frontend] openProjectInEditor', { path, editorName });
  try {
    await invoke<void>('open_project_in_editor', { path, editorName });
  } catch (e) {
    console.error('[vori][frontend] openProjectInEditor FAILED', e);
    throw e;
  }
};

export const openWorkspaceInEditor = async (paths: string[], editorName: string) => {
  console.info('[vori][frontend] openWorkspaceInEditor', { paths, editorName });
  try {
    await invoke<void>('open_workspace_in_editor', { paths, editorName });
  } catch (e) {
    console.error('[vori][frontend] openWorkspaceInEditor FAILED', e);
    throw e;
  }
};

export const openFileInEditor = async (path: string, textEditor?: string) => {
  console.info('[vori][frontend] openFileInEditor', { path, textEditor });
  try {
    await invoke<void>('open_file_in_editor', { path, textEditor });
  } catch (e) {
    console.error('[vori][frontend] openFileInEditor FAILED', e);
    throw e;
  }
};

export const openInTerminal = async (path?: string) => {
  console.info('[vori][frontend] openInTerminal', { path });
  try {
    await invoke<void>('open_in_terminal', { path });
  } catch (e) {
    console.error('[vori][frontend] openInTerminal FAILED', e);
    throw e;
  }
};

export const detectTerminals = () =>
  invoke<Record<string, string>>('detect_terminals');

export const detectEditors = () =>
  invoke<Record<string, string>>('detect_editors');

export const getInstalledApps = () =>
  invoke<{ name: string; exec: string }[]>('get_installed_apps');

// ── Scanner ───────────────────────────────────────────────────────────────────

export const scanFolder = (path: string, maxDepth?: number) =>
  invoke<ScannedProject[]>('scan_folder', { path, maxDepth });

export const detectWorkspacesInFolder = (path: string, maxDepth?: number) =>
  invoke<DetectedWorkspace[]>('detect_workspaces_in_folder', { path, maxDepth });

// ── Search ────────────────────────────────────────────────────────────────────

export const search = (query: string) => invoke<SearchResult[]>('search', { query });

// ── Git ───────────────────────────────────────────────────────────────────────

/** Git state of every given path that is a repository (others are absent). */
export const getGitInfo = (paths: string[]) =>
  invoke<Record<string, GitInfo>>('get_git_info', { paths });

/** Clone `url` into `<destParent>/<repo name>`; resolves to the new folder. */
export const gitClone = (url: string, destParent: string) =>
  invoke<string>('git_clone', { url, destParent });

// ── CLI / deep links ──────────────────────────────────────────────────────────

/** Requests queued by the command line or a vori:// link since the last call. */
export const takePendingRequests = () => invoke<CliRequest[]>('take_pending_requests');
