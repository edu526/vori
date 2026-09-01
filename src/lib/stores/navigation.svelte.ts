import type { CategoriesMap, DetectedWorkspace, Favorites, FilesMap, ProjectsMap, RecentItem } from '$lib/api/types';
import {
  detectWorkspacesInFolder,
  getWorkspaceSelection,
  setWorkspaceSelection,
} from '$lib/api/commands';

export type NavItemType = 'category' | 'project' | 'file' | 'workspace';

export interface NavItem {
  key: string;
  label: string;
  type: NavItemType;
  path?: string;
  parentKey?: string | null;
  hasChildren?: boolean;
  isFavorite?: boolean;
  stack?: string;
}

export interface Column {
  items: NavItem[];
  selectedKey: string | null;
  title?: string;
}

function itemHasChildren(key: string, categories: CategoriesMap, projects: ProjectsMap, files: FilesMap): boolean {
  return (
    Object.values(categories).some((c) => c.parent === key) ||
    Object.values(projects).some((p) => p.parent === key) ||
    Object.values(files).some((f) => f.parent === key)
  );
}

function buildRootItems(
  categories: CategoriesMap,
  projects: ProjectsMap,
  files: FilesMap,
  favorites: Favorites,
): NavItem[] {
  const items: NavItem[] = [];

  // Root categories (parent === null)
  for (const [key, cat] of Object.entries(categories).sort(([a], [b]) => a.localeCompare(b))) {
    if (cat.parent === null) {
      items.push({
        key,
        label: key.split('/').pop() ?? key,
        type: 'category',
        parentKey: null,
        hasChildren: itemHasChildren(key, categories, projects, files),
        isFavorite: favorites.categories.includes(key),
      });
    }
  }

  // Root files: either parent is null/missing, or parent points to a
  // category that no longer exists (orphan files fallback to root).
  for (const [key, file] of Object.entries(files).sort(([a], [b]) => a.localeCompare(b))) {
    if (!file.parent || !categories[file.parent]) {
      items.push({
        key,
        label: key,
        type: 'file',
        path: file.path,
        isFavorite: favorites.files.includes(key),
      });
    }
  }

  return items;
}

function buildChildItems(
  parentKey: string,
  categories: CategoriesMap,
  projects: ProjectsMap,
  files: FilesMap,
  favorites: Favorites,
): NavItem[] {
  const items: NavItem[] = [];

  // Child categories
  for (const [key, cat] of Object.entries(categories).sort(([a], [b]) => a.localeCompare(b))) {
    if (cat.parent === parentKey) {
      items.push({
        key,
        label: key.split('/').pop() ?? key,
        type: 'category',
        parentKey,
        hasChildren: itemHasChildren(key, categories, projects, files),
        isFavorite: favorites.categories.includes(key),
      });
    }
  }

  // Projects belonging to this parent
  for (const [key, proj] of Object.entries(projects).sort(([a], [b]) => a.localeCompare(b))) {
    if (proj.parent === parentKey) {
      items.push({
        key,
        label: proj.path.split(/[\\/]/).pop() ?? key,
        type: 'project',
        path: proj.path,
        parentKey,
        isFavorite: favorites.projects.includes(key),
        stack: proj.stack,
      });
    }
  }

  // Files belonging to this parent
  for (const [key, file] of Object.entries(files).sort(([a], [b]) => a.localeCompare(b))) {
    if (file.parent === parentKey) {
      items.push({
        key,
        label: key,
        type: 'file',
        path: file.path,
        isFavorite: favorites.files.includes(key),
      });
    }
  }

  return items;
}

function selectableItems(col: Column): NavItem[] {
  return col.items;
}

export interface WorkspaceEntry {
  path: string;
  label: string;
}

function createNavigationStore() {
  let columns = $state<Column[]>([]);
  let activeColumnIndex = $state(0);
  let _categories = $state<CategoriesMap>({});
  let _projects = $state<ProjectsMap>({});
  let _files = $state<FilesMap>({});
  let _favorites = $state<Favorites>({ projects: [], files: [], categories: [] });
  let _recents = $state<RecentItem[]>([]);
  let _workspaceSelection = $state<Map<string, WorkspaceEntry>>(new Map());
  let _workspaceDetections = $state<Record<string, DetectedWorkspace[]>>({});
  let _activeCategoryKey = $state<string | null>(null);

  function init(
    cats: CategoriesMap,
    projs: ProjectsMap,
    fls: FilesMap,
    favs: Favorites,
    recents: RecentItem[],
  ) {
    _categories = cats;
    _projects = projs;
    _files = fls;
    _favorites = favs;
    _recents = recents;
    activeColumnIndex = 0;
    columns = [
      {
        items: buildRootItems(cats, projs, fls, favs),
        selectedKey: null,
        title: 'Projects',
      },
    ];
  }

  function selectItem(columnIndex: number, key: string) {
    const item = columns[columnIndex]?.items.find((it) => it.key === key);
    if (!item) return;

    activeColumnIndex = columnIndex;

    columns = columns
      .slice(0, columnIndex + 1)
      .map((col, i) => (i === columnIndex ? { ...col, selectedKey: key } : col));

    if (item.type === 'category') {
      _activeCategoryKey = key;
      const nextItems = buildChildItems(key, _categories, _projects, _files, _favorites);
      const wsItems = workspaceItemsFor(key);
      columns = [...columns, {
        items: [...nextItems, ...wsItems],
        selectedKey: null,
        title: key.split('/').pop() ?? key,
      }];

      const sourcePath = _categories[key]?.source_path;
      if (sourcePath && _workspaceDetections[key] === undefined) {
        detectWorkspaces(key, sourcePath);
      }
    } else {
      _activeCategoryKey = null;
    }
  }

  function workspaceItemsFor(key: string): NavItem[] {
    return (_workspaceDetections[key] ?? []).map((ws) => ({
      key: `__ws:${ws.workspace_file}`,
      label: ws.name,
      type: 'workspace',
      path: ws.workspace_file,
      parentKey: key,
    }));
  }

  function detectWorkspaces(key: string, sourcePath: string) {
    detectWorkspacesInFolder(sourcePath)
      .then((workspaces) => {
        _workspaceDetections = { ..._workspaceDetections, [key]: workspaces };
        if (_activeCategoryKey !== key) return;
        const colIdx = columns.length - 1;
        if (colIdx <= 0) return;
        const baseItems = buildChildItems(key, _categories, _projects, _files, _favorites);
        const wsItems = workspaceItemsFor(key);
        columns = [
          ...columns.slice(0, colIdx),
          { ...columns[colIdx], items: [...baseItems, ...wsItems] },
        ];
      })
      .catch((err) => {
        console.warn('[vori] workspace detection failed for', key, err);
      });
  }

  // ── Keyboard navigation ───────────────────────────────────────────────────

  function moveSelection(delta: -1 | 1) {
    const col = columns[activeColumnIndex];
    if (!col) return;

    const items = selectableItems(col);
    if (items.length === 0) return;

    const currentIdx = items.findIndex((it) => it.key === col.selectedKey);
    let nextIdx: number;

    if (currentIdx === -1) {
      nextIdx = delta === 1 ? 0 : items.length - 1;
    } else {
      nextIdx = Math.max(0, Math.min(items.length - 1, currentIdx + delta));
    }

    selectItem(activeColumnIndex, items[nextIdx].key);
  }

  function expandRight() {
    const col = columns[activeColumnIndex];
    if (!col) return;

    // Nothing selected — select first item in current column
    if (!col.selectedKey) {
      const first = selectableItems(col)[0];
      if (first) selectItem(activeColumnIndex, first.key);
      return;
    }

    // Next column exists — move into it and select first item
    const nextCol = columns[activeColumnIndex + 1];
    if (nextCol) {
      activeColumnIndex = activeColumnIndex + 1;
      const first = selectableItems(nextCol)[0];
      if (first) selectItem(activeColumnIndex, first.key);
    }
    // Leaf node (project/file) — Enter handles opening
  }

  function collapseLeft() {
    if (activeColumnIndex > 0) {
      activeColumnIndex = activeColumnIndex - 1;
    } else {
      columns = columns.slice(0, 1).map((col) => ({ ...col, selectedKey: null }));
    }
  }

  // Deselect the deepest selected column and remove its children.
  // Returns true if something was collapsed, false if already at root with nothing selected.
  function collapseDeepest(): boolean {
    for (let i = columns.length - 1; i >= 0; i--) {
      if (columns[i].selectedKey !== null) {
        columns = columns.slice(0, i + 1).map((col, idx) =>
          idx === i ? { ...col, selectedKey: null } : col,
        );
        activeColumnIndex = Math.min(activeColumnIndex, i);
        return true;
      }
    }
    return false;
  }

  function collapseToDepth(depth: number) {
    if (depth < 0) return;
    columns = columns.slice(0, depth + 1);
    activeColumnIndex = Math.min(activeColumnIndex, depth);
  }

  function refresh(
    cats: CategoriesMap,
    projs: ProjectsMap,
    fls: FilesMap,
    favs: Favorites,
    recents: RecentItem[],
  ) {
    _categories = cats;
    _projects = projs;
    _files = fls;
    _favorites = favs;
    _recents = recents;
    const rootSelectedKey = columns[0]?.selectedKey ?? null;
    columns = [
      {
        items: buildRootItems(cats, projs, fls, favs),
        selectedKey: rootSelectedKey,
        title: 'Projects',
      },
    ];
    if (rootSelectedKey) {
      selectItem(0, rootSelectedKey);
    }
  }

  // Prepend a recent to the root column without losing current navigation
  function addRecentToView(item: RecentItem) {
    _recents = [item, ..._recents.filter((r) => r.path !== item.path)].slice(0, 20);
    const rootSelectedKey = columns[0]?.selectedKey ?? null;
    columns = [
      {
        items: buildRootItems(_categories, _projects, _files, _favorites),
        selectedKey: rootSelectedKey,
        title: 'Projects',
      },
      ...columns.slice(1),
    ];
  }

  // Patch isFavorite on every item across all columns without rebuilding
  function updateFavorites(favs: Favorites) {
    _favorites = favs;
    columns = columns.map((col) => ({
      ...col,
      items: col.items.map((item) => {
        if (item.type === 'project') {
          return { ...item, isFavorite: favs.projects.includes(item.key) };
        } else if (item.type === 'file') {
          // Recent files use key "__r:name"; regular files use key = name
          const fileKey = item.key.startsWith('__r:') ? item.key.slice(4) : item.key;
          return { ...item, isFavorite: favs.files.includes(fileKey) };
        } else if (item.type === 'category') {
          return { ...item, isFavorite: favs.categories.includes(item.key) };
        }
        return item;
      }),
    }));
  }

  function toggleWorkspaceItem(key: string, path: string, label: string) {
    // ponytail: dedup by path so the selection survives project-key renames
    // and is consistent for both project and workspace items.
    const next = new Map(_workspaceSelection);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.set(path, { path, label });
    }
    _workspaceSelection = next;
    void setWorkspaceSelection([...next.values()]);
  }

  function clearWorkspaceSelection() {
    _workspaceSelection = new Map();
    void setWorkspaceSelection([]);
  }

  async function loadWorkspaceSelection() {
    const entries = await getWorkspaceSelection();
    const map = new Map<string, WorkspaceEntry>();
    for (const e of entries) {
      map.set(e.path, e);
    }
    _workspaceSelection = map;
  }

  return {
    get columns() {
      return columns;
    },
    get activeColumnIndex() {
      return activeColumnIndex;
    },
    get workspaceSelection() {
      return _workspaceSelection;
    },
    get workspaceDetections() {
      return _workspaceDetections;
    },
    get selectedItem(): NavItem | null {
      for (let i = columns.length - 1; i >= 0; i--) {
        if (columns[i].selectedKey) {
          const item = columns[i].items.find((it) => it.key === columns[i].selectedKey);
          if (item) return item;
        }
      }
      return null;
    },
    init,
    selectItem,
    moveSelection,
    expandRight,
    collapseLeft,
    collapseDeepest,
    collapseToDepth,
    refresh,
    updateFavorites,
    addRecentToView,
    toggleWorkspaceItem,
    clearWorkspaceSelection,
    loadWorkspaceSelection,
    detectWorkspaces,
  };
}

export const navigationStore = createNavigationStore();
