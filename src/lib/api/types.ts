// ── Category types ────────────────────────────────────────────────────────────

export interface Subcategory {
  description: string;
  icon: string;
}

export interface Category {
  description: string;
  icon: string;
  subcategories: Record<string, Subcategory>;
}

export type CategoriesMap = Record<string, Category>;

// ── Project types ─────────────────────────────────────────────────────────────

export interface Project {
  path: string;
  category: string;
  subcategory?: string;
}

export type ProjectsMap = Record<string, Project>;

// ── File types ────────────────────────────────────────────────────────────────

export interface FileEntry {
  path: string;
}

export type FilesMap = Record<string, FileEntry>;

// ── Preferences ───────────────────────────────────────────────────────────────

export interface TerminalPreferences {
  preferred?: string;
  available: Record<string, string>;
  last_detected?: string;
}

export type Theme = 'system' | 'light' | 'dark';

export interface Preferences {
  default_editor: string;
  default_text_editor?: string;
  close_on_open_editor: boolean;
  close_on_open_terminal: boolean;
  close_on_open_file: boolean;
  terminal: TerminalPreferences;
  editors_available: Record<string, string>;
  theme: Theme;
}

// ── Favorites ─────────────────────────────────────────────────────────────────

export interface Favorites {
  projects: string[];
  files: string[];
  categories: string[];
}

// ── Recents ───────────────────────────────────────────────────────────────────

export type RecentType = 'project' | 'file' | 'category';

export interface RecentItem {
  path: string;
  name: string;
  type: RecentType;
  timestamp: number;
}

// ── Combined app data ─────────────────────────────────────────────────────────

export interface AppData {
  categories: CategoriesMap;
  projects: ProjectsMap;
  files: FilesMap;
  preferences: Preferences;
  favorites: Favorites;
  recents: RecentItem[];
}

// ── Search ────────────────────────────────────────────────────────────────────

export interface SearchResult {
  key: string;
  name: string;
  result_type: string;
  path?: string;
  category?: string;
  subcategory?: string;
}
