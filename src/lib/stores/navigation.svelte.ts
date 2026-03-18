import type { CategoriesMap, Favorites, FilesMap, ProjectsMap, RecentItem } from '$lib/api/types';

export type NavItemType = 'category' | 'subcategory' | 'project' | 'file' | 'section-header';

export interface NavItem {
  key: string;
  label: string;
  type: NavItemType;
  path?: string;
  categoryKey?: string;
  subcategoryKey?: string;
  hasChildren?: boolean;
  isFavorite?: boolean;
}

export interface Column {
  items: NavItem[];
  selectedKey: string | null;
  title?: string;
}

const MAX_RECENTS_IN_COLUMN = 5;

function buildRootItems(
  categories: CategoriesMap,
  projects: ProjectsMap,
  files: FilesMap,
  favorites: Favorites,
  recents: RecentItem[],
): NavItem[] {
  const items: NavItem[] = [];

  // ── Recents ───────────────────────────────────────────────────────────────
  const recentItems = recents
    .filter((r) => r.type === 'project' || r.type === 'file')
    .slice(0, MAX_RECENTS_IN_COLUMN);

  if (recentItems.length > 0) {
    items.push({ key: '__recents__', label: 'Recent', type: 'section-header' });
    for (const recent of recentItems) {
      const key = recent.type === 'file' ? `__r:${recent.name}` : recent.name;
      items.push({
        key,
        label: recent.name,
        type: recent.type as 'project' | 'file',
        path: recent.path,
        isFavorite:
          recent.type === 'project'
            ? favorites.projects.includes(recent.name)
            : favorites.files.includes(recent.name),
      });
    }
  }

  // ── Categories ────────────────────────────────────────────────────────────
  if (Object.keys(categories).length > 0) {
    items.push({ key: '__categories__', label: 'Categories', type: 'section-header' });
  }
  for (const [key] of Object.entries(categories)) {
    items.push({
      key,
      label: key,
      type: 'category',
      hasChildren: true,
      isFavorite: favorites.categories.includes(key),
    });
  }

  // ── Files ─────────────────────────────────────────────────────────────────
  if (Object.keys(files).length > 0) {
    items.push({ key: '__files__', label: 'Files', type: 'section-header' });
    for (const [key, file] of Object.entries(files)) {
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

function buildCategoryItems(
  categoryKey: string,
  categories: CategoriesMap,
  projects: ProjectsMap,
  favorites: Favorites,
): NavItem[] {
  const cat = categories[categoryKey];
  if (!cat) return [];

  const items: NavItem[] = [];
  const hasSubcats = Object.keys(cat.subcategories).length > 0;

  if (hasSubcats) {
    items.push({ key: '__subcats__', label: 'Subcategories', type: 'section-header' });
    for (const [key] of Object.entries(cat.subcategories)) {
      items.push({ key, label: key, type: 'subcategory', categoryKey, hasChildren: true });
    }
  }

  const directProjects = Object.entries(projects).filter(
    ([, p]) => p.category === categoryKey && !p.subcategory,
  );

  if (directProjects.length > 0) {
    items.push({ key: '__projects__', label: 'Projects', type: 'section-header' });
    for (const [key, proj] of directProjects) {
      items.push({
        key,
        label: key,
        type: 'project',
        path: proj.path,
        categoryKey,
        isFavorite: favorites.projects.includes(key),
      });
    }
  }

  return items;
}

function buildSubcategoryItems(
  categoryKey: string,
  subcategoryKey: string,
  projects: ProjectsMap,
  favorites: Favorites,
): NavItem[] {
  return Object.entries(projects)
    .filter(([, p]) => p.category === categoryKey && p.subcategory === subcategoryKey)
    .map(([key, proj]) => ({
      key,
      label: key,
      type: 'project' as NavItemType,
      path: proj.path,
      categoryKey,
      subcategoryKey,
      isFavorite: favorites.projects.includes(key),
    }));
}

function selectableItems(col: Column): NavItem[] {
  return col.items.filter((it) => it.type !== 'section-header');
}

function createNavigationStore() {
  let columns = $state<Column[]>([]);
  let activeColumnIndex = $state(0);
  let _categories = $state<CategoriesMap>({});
  let _projects = $state<ProjectsMap>({});
  let _files = $state<FilesMap>({});
  let _favorites = $state<Favorites>({ projects: [], files: [], categories: [] });
  let _recents = $state<RecentItem[]>([]);

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
        items: buildRootItems(cats, projs, fls, favs, recents),
        selectedKey: null,
        title: 'Projects',
      },
    ];
  }

  function selectItem(columnIndex: number, key: string) {
    const item = columns[columnIndex]?.items.find((it) => it.key === key);
    if (!item || item.type === 'section-header') return;

    activeColumnIndex = columnIndex;

    columns = columns
      .slice(0, columnIndex + 1)
      .map((col, i) => (i === columnIndex ? { ...col, selectedKey: key } : col));

    let nextItems: NavItem[] = [];
    let nextTitle: string | undefined;

    if (item.type === 'category') {
      nextItems = buildCategoryItems(key, _categories, _projects, _favorites);
      nextTitle = key;
    } else if (item.type === 'subcategory' && item.categoryKey) {
      nextItems = buildSubcategoryItems(item.categoryKey, key, _projects, _favorites);
      nextTitle = key;
    }

    if (nextItems.length > 0) {
      columns = [...columns, { items: nextItems, selectedKey: null, title: nextTitle }];
    }
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
    const nextCol = columns[activeColumnIndex + 1];
    if (!nextCol) return;
    const first = selectableItems(nextCol)[0];
    if (first) {
      activeColumnIndex = activeColumnIndex + 1;
      selectItem(activeColumnIndex, first.key);
    } else {
      activeColumnIndex = activeColumnIndex + 1;
    }
  }

  function collapseLeft() {
    if (activeColumnIndex > 0) {
      activeColumnIndex = activeColumnIndex - 1;
    }
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
        items: buildRootItems(cats, projs, fls, favs, recents),
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
        items: buildRootItems(_categories, _projects, _files, _favorites, _recents),
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
      items: col.items.map((item) => ({
        ...item,
        isFavorite:
          item.type === 'project'
            ? favs.projects.includes(item.key)
            : item.type === 'file'
              ? favs.files.includes(item.key)
              : item.type === 'category'
                ? favs.categories.includes(item.key)
                : item.isFavorite,
      })),
    }));
  }

  return {
    get columns() {
      return columns;
    },
    get activeColumnIndex() {
      return activeColumnIndex;
    },
    get selectedItem(): NavItem | null {
      for (let i = columns.length - 1; i >= 0; i--) {
        if (columns[i].selectedKey) {
          const item = columns[i].items.find((it) => it.key === columns[i].selectedKey);
          if (item && item.type !== 'section-header') return item;
        }
      }
      return null;
    },
    init,
    selectItem,
    moveSelection,
    expandRight,
    collapseLeft,
    refresh,
    updateFavorites,
    addRecentToView,
  };
}

export const navigationStore = createNavigationStore();
