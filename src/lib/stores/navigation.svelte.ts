import type { CategoriesMap, FilesMap, ProjectsMap } from '$lib/api/types';

export type NavItemType = 'category' | 'subcategory' | 'project' | 'file' | 'section-header';

export interface NavItem {
  key: string;
  label: string;
  type: NavItemType;
  path?: string;
  categoryKey?: string;
  subcategoryKey?: string;
  hasChildren?: boolean;
}

export interface Column {
  items: NavItem[];
  selectedKey: string | null;
  title?: string;
}

function buildRootItems(categories: CategoriesMap, files: FilesMap): NavItem[] {
  const items: NavItem[] = [];

  for (const [key] of Object.entries(categories)) {
    items.push({ key, label: key, type: 'category', hasChildren: true });
  }

  if (Object.keys(files).length > 0) {
    items.push({ key: '__files__', label: 'Files', type: 'section-header' });
    for (const [key, file] of Object.entries(files)) {
      items.push({ key, label: key, type: 'file', path: file.path });
    }
  }

  return items;
}

function buildCategoryItems(
  categoryKey: string,
  categories: CategoriesMap,
  projects: ProjectsMap,
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
      items.push({ key, label: key, type: 'project', path: proj.path, categoryKey });
    }
  }

  return items;
}

function buildSubcategoryItems(
  categoryKey: string,
  subcategoryKey: string,
  projects: ProjectsMap,
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
    }));
}

function createNavigationStore() {
  let columns = $state<Column[]>([]);
  let _categories = $state<CategoriesMap>({});
  let _projects = $state<ProjectsMap>({});
  let _files = $state<FilesMap>({});

  function init(cats: CategoriesMap, projs: ProjectsMap, fls: FilesMap) {
    _categories = cats;
    _projects = projs;
    _files = fls;
    columns = [{ items: buildRootItems(cats, fls), selectedKey: null, title: 'Projects' }];
  }

  function selectItem(columnIndex: number, key: string) {
    // Section headers are not selectable
    const item = columns[columnIndex]?.items.find((it) => it.key === key);
    if (!item || item.type === 'section-header') return;

    // Update selection, drop all columns after this one
    columns = columns
      .slice(0, columnIndex + 1)
      .map((col, i) => (i === columnIndex ? { ...col, selectedKey: key } : col));

    // Push next column if the item has children
    let nextItems: NavItem[] = [];
    let nextTitle: string | undefined;

    if (item.type === 'category') {
      nextItems = buildCategoryItems(key, _categories, _projects);
      nextTitle = key;
    } else if (item.type === 'subcategory' && item.categoryKey) {
      nextItems = buildSubcategoryItems(item.categoryKey, key, _projects);
      nextTitle = key;
    }

    if (nextItems.length > 0) {
      columns = [...columns, { items: nextItems, selectedKey: null, title: nextTitle }];
    }
  }

  function refresh(cats: CategoriesMap, projs: ProjectsMap, fls: FilesMap) {
    _categories = cats;
    _projects = projs;
    _files = fls;
    // Rebuild root column, preserve selection
    const rootSelectedKey = columns[0]?.selectedKey ?? null;
    columns = [{ items: buildRootItems(cats, fls), selectedKey: rootSelectedKey, title: 'Projects' }];
    // Re-apply selection to rebuild child columns
    if (rootSelectedKey) {
      selectItem(0, rootSelectedKey);
    }
  }

  return {
    get columns() {
      return columns;
    },
    init,
    selectItem,
    refresh,
  };
}

export const navigationStore = createNavigationStore();
