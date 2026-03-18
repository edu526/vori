import { getAppData } from '$lib/api/commands';
import type {
  AppData,
  CategoriesMap,
  Favorites,
  FilesMap,
  Preferences,
  ProjectsMap,
  RecentItem,
} from '$lib/api/types';

function createConfigStore() {
  let categories = $state<CategoriesMap>({});
  let projects = $state<ProjectsMap>({});
  let files = $state<FilesMap>({});
  let preferences = $state<Preferences>({
    default_editor: 'vscode',
    close_on_open: false,
    terminal: { available: {} },
    editors_available: {},
  });
  let favorites = $state<Favorites>({ projects: [], files: [], categories: [] });
  let recents = $state<RecentItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function load() {
    try {
      loading = true;
      error = null;
      const data: AppData = await getAppData();
      categories = data.categories;
      projects = data.projects;
      files = data.files;
      preferences = data.preferences;
      favorites = data.favorites;
      recents = data.recents;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  return {
    get categories() {
      return categories;
    },
    set categories(v: CategoriesMap) {
      categories = v;
    },
    get projects() {
      return projects;
    },
    set projects(v: ProjectsMap) {
      projects = v;
    },
    get files() {
      return files;
    },
    set files(v: FilesMap) {
      files = v;
    },
    get preferences() {
      return preferences;
    },
    set preferences(v: Preferences) {
      preferences = v;
    },
    get favorites() {
      return favorites;
    },
    set favorites(v: Favorites) {
      favorites = v;
    },
    get recents() {
      return recents;
    },
    set recents(v: RecentItem[]) {
      recents = v;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    load,
  };
}

export const configStore = createConfigStore();
