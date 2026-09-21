import { ask, message, open } from '@tauri-apps/plugin-dialog';
import { t, tn } from '$lib/i18n/index.svelte';
import type { NavItem } from '$lib/stores/navigation.svelte';
import { navigationStore } from '$lib/stores/navigation.svelte';
import { syncStore } from '$lib/stores/sync.svelte';
import { configStore } from '$lib/stores/config.svelte';
import { dialogStore } from '$lib/stores/dialogs.svelte';
import { openWorkspaceInEditor, runProjectScript } from '$lib/api/commands';
import type { Favorites, ProjectScripts } from '$lib/api/types';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { copyText } from '$lib/utils/clipboard';
import type { MenuItem } from '$lib/stores/contextMenu.svelte';
import {
  openProjectInEditor,
  openInTerminal,
  openFileInEditor,
  addProject,
  bulkImportProjects,
  updateCategory,
  deleteCategory,
  deleteProject,
  deleteFile,
  toggleFavorite,
} from '$lib/api/commands';

const EDITOR_LABELS: Record<string, string> = {
  'vscode': 'VSCode', 'vscode-insiders': 'VSCode Insiders', 'cursor': 'Cursor',
  'windsurf': 'Windsurf', 'kiro': 'Kiro', 'zed': 'Zed', 'fleet': 'Fleet',
  'sublime': 'Sublime Text', 'graviton': 'Graviton', 'helix': 'Helix',
  'neovim': 'Neovim', 'vim': 'Vim', 'emacs': 'Emacs', 'kate': 'Kate', 'gedit': 'Gedit',
};

function editorLabel(key: string): string {
  return EDITOR_LABELS[key] ?? key;
}

const MAX_MENU_SCRIPTS = 6;

async function runScript(path: string, script: string) {
  try {
    const result = await runProjectScript(path, script);
    if (!result.ran) {
      // The terminal can't be told to run a command: it was opened in the folder instead.
      const copied = await copyText(result.command);
      await message(
        copied
          ? t('script.terminalCantRun', { command: result.command })
          : t('script.terminalCantRunManual', { command: result.command }),
        { title: t('script.runTitle'), kind: 'info' },
      );
    }
  } catch (e) {
    await message(String(e), { title: t('script.runFailedTitle'), kind: 'error' });
  }
}

export function buildMenuItems(
  item: NavItem,
  opts: {
    defaultEditor: string;
    editorsAvailable: Record<string, string>;
    favorites: Favorites;
    onEdit: () => void;
    onRefresh: () => void;
    onAddChildCategory?: () => void;
    onAddProject?: () => void;
    onImportFolder?: (autoScanPath?: string) => void;
    onCloneRepo?: () => void;
    /** `package.json` scripts of the project, when the item is a Node project. */
    scripts?: ProjectScripts | null;
  },
): MenuItem[] {
  const primaryLabel = editorLabel(opts.defaultEditor);
  const otherEditors = Object.keys(opts.editorsAvailable)
    .filter((k) => k !== opts.defaultEditor)
    .sort((a, b) => a.localeCompare(b));

  switch (item.type) {
    case 'category': {
      const cat = configStore.categories[item.key];
      const existingSourcePath = cat?.source_path;
      const detected = navigationStore.workspaceDetections[item.key] ?? [];
      const newFolders = existingSourcePath ? syncStore.newCount(item.key) : 0;
      return [
        {
          label: t('menu.addSubcategory'),
          action: () => opts.onAddChildCategory?.(),
        },
        {
          label: t('menu.addProjectHere'),
          action: () => opts.onAddProject?.(),
        },
        {
          label: t('menu.cloneRepo'),
          action: () => opts.onCloneRepo?.(),
        },
        {
          label: t('menu.importFolder'),
          action: () => opts.onImportFolder?.(),
        },
        ...(newFolders > 0
          ? [
              {
                label: tn('menu.importNew.one', 'menu.importNew.other', newFolders),
                action: () => opts.onImportFolder?.(existingSourcePath ?? undefined),
              },
              {
                label: t('menu.ignoreNew'),
                action: () => syncStore.dismiss(item.key),
              },
            ]
          : []),
        ...(existingSourcePath
          ? [
              {
                label: t('menu.refreshImport'),
                action: () => opts.onImportFolder?.(existingSourcePath),
              },
            ]
          : []),
        { label: '', action: () => {}, divider: true },
        {
          label: existingSourcePath ? t('menu.detectWorkspaces') : t('menu.detectWorkspacesIn'),
          action: async () => {
            let sourcePath = existingSourcePath;
            if (!sourcePath) {
              const picked = await open({ directory: true });
              if (typeof picked !== 'string') return;
              sourcePath = picked;
              await updateCategory(item.key, cat?.parent ?? null, sourcePath);
              await configStore.load();
              opts.onRefresh();
            }
            navigationStore.detectWorkspaces(item.key, sourcePath);
          },
        },
        ...(detected.length > 0
          ? [
              {
                label: tn('menu.importWorkspaces.one', 'menu.importWorkspaces.other', detected.length),
                action: async () => {
                  const used = new Set(Object.keys(configStore.projects));
                  const entries: [string, { path: string; parent: string }][] = detected.map((ws) => {
                    let key = `ws-${ws.name}`;
                    if (used.has(key)) {
                      const rel = ws.relative_path.replace(/\//g, '-');
                      key = used.has(`ws-${rel}`) ? `ws-${ws.workspace_file}` : `ws-${rel}`;
                    }
                    used.add(key);
                    return [key, { path: ws.workspace_file, parent: item.key }];
                  });
                  await bulkImportProjects(entries);
                  await configStore.load();
                  opts.onRefresh();
                },
              },
            ]
          : []),
        { label: '', action: () => {}, divider: true },
        {
          label: t('menu.editCategory'),
          action: () => opts.onEdit(),
        },
        {
          label: t('menu.deleteCategory'),
          danger: true,
          action: async () => {
            const ok = await ask(t('menu.confirmDeleteCategory', { name: item.label }), { kind: 'warning' });
            if (ok) await deleteCategory(item.key).then(() => opts.onRefresh());
          },
        },
      ];
    }

    case 'project': {
      const isFav = opts.favorites.projects.includes(item.key);
      const inWorkspace = !!item.path && navigationStore.workspaceSelection.has(item.path);
      const workspaceSize = navigationStore.workspaceSelection.size;
      return [
        {
          label: t('common.openIn', { editor: primaryLabel }),
          action: () => openProjectInEditor(item.path!, opts.defaultEditor),
        },
        ...otherEditors.map((key) => ({
          label: t('common.openIn', { editor: editorLabel(key) }),
          action: () => openProjectInEditor(item.path!, key),
        })),
        {
          label: t('menu.openInTerminal'),
          action: () => openInTerminal(item.path!),
        },
        ...(opts.scripts && opts.scripts.scripts.length > 0
          ? [
              { label: '', action: () => {}, divider: true },
              ...opts.scripts.scripts.slice(0, MAX_MENU_SCRIPTS).map((script) => ({
                label: t('menu.runScript', { script }),
                action: () => runScript(item.path!, script),
              })),
            ]
          : []),
        { label: '', action: () => {}, divider: true },
        {
          label: t('menu.showInFileManager'),
          action: () => revealItemInDir(item.path!),
        },
        {
          label: t('menu.copyPath'),
          action: () => { void copyText(item.path!); },
        },
        { label: '', action: () => {}, divider: true },
        {
          label: inWorkspace ? t('menu.removeFromWorkspace') : t('menu.addToWorkspace'),
          action: () => navigationStore.toggleWorkspaceItem(item.key, item.path!, item.label),
        },
        ...(workspaceSize >= 2 && inWorkspace || workspaceSize >= 1 && !inWorkspace
          ? [{
              label: t('menu.openProjectsAsWorkspace', { count: workspaceSize + (inWorkspace ? 0 : 1) }),
              action: async () => {
                if (!inWorkspace) {
                  navigationStore.toggleWorkspaceItem(item.key, item.path!, item.label);
                }
                const paths = [...navigationStore.workspaceSelection.values()].map((v) => v.path);
                await openWorkspaceInEditor(paths, opts.defaultEditor);
                navigationStore.clearWorkspaceSelection();
              },
            }]
          : []),
        { label: '', action: () => {}, divider: true },
        {
          label: isFav ? t('menu.removeFavorite') : t('menu.addFavorite'),
          action: () => {
            toggleFavorite(item.key, 'project').then((favs) => { navigationStore.updateFavorites(favs); configStore.favorites = favs; });
          },
        },
        { label: '', action: () => {}, divider: true },
        {
          label: t('menu.editProject'),
          action: () => opts.onEdit(),
        },
        {
          label: t('menu.deleteProject'),
          danger: true,
          action: async () => {
            const ok = await ask(t('menu.confirmDeleteProject', { name: item.label }), { kind: 'warning' });
            if (ok) await deleteProject(item.key).then(() => opts.onRefresh());
          },
        },
      ];
    }

    case 'workspace': {
      // ponytail: detected workspaces are ephemeral — no favorites, no edit, no delete.
      // User can "Import as project" to make one persistent.
      const inWorkspace = !!item.path && navigationStore.workspaceSelection.has(item.path);
      const workspaceSize = navigationStore.workspaceSelection.size;
      return [
        {
          label: t('common.openIn', { editor: primaryLabel }),
          action: () => openProjectInEditor(item.path!, opts.defaultEditor),
        },
        ...otherEditors.map((key) => ({
          label: t('common.openIn', { editor: editorLabel(key) }),
          action: () => openProjectInEditor(item.path!, key),
        })),
        { label: '', action: () => {}, divider: true },
        {
          label: inWorkspace ? t('menu.removeFromWorkspace') : t('menu.addToWorkspace'),
          action: () => navigationStore.toggleWorkspaceItem(item.key, item.path!, item.label),
        },
        ...(workspaceSize >= 2 && inWorkspace || workspaceSize >= 1 && !inWorkspace
          ? [{
              label: t('menu.openAsWorkspace', { count: workspaceSize + (inWorkspace ? 0 : 1) }),
              action: async () => {
                if (!inWorkspace) {
                  navigationStore.toggleWorkspaceItem(item.key, item.path!, item.label);
                }
                const paths = [...navigationStore.workspaceSelection.values()].map((v) => v.path);
                await openWorkspaceInEditor(paths, opts.defaultEditor);
                navigationStore.clearWorkspaceSelection();
              },
            }]
          : []),
        { label: '', action: () => {}, divider: true },
        {
          label: t('menu.importAsProject'),
          action: async () => {
            let key = `ws-${item.label}`;
            if (configStore.projects[key]) {
              key = `ws-${item.path!.split(/[\\/]/).pop()?.replace(/\.code-workspace$/, '') ?? item.label}`;
            }
            await addProject(key, { path: item.path!, parent: item.parentKey ?? '' });
            await configStore.load();
            opts.onRefresh();
          },
        },
      ];
    }

    case 'file': {
      const isFav = opts.favorites.files.includes(item.key);
      return [
        { label: '', action: () => {}, divider: true },
        {
          label: isFav ? t('menu.removeFavorite') : t('menu.addFavorite'),
          action: () => {
            toggleFavorite(item.key, 'file').then((favs) => { navigationStore.updateFavorites(favs); configStore.favorites = favs; });
          },
        },
        { label: '', action: () => {}, divider: true },
        {
          label: t('menu.editFile'),
          action: () => opts.onEdit(),
        },
        {
          label: t('menu.deleteFile'),
          danger: true,
          action: async () => {
            const ok = await ask(t('menu.confirmDeleteFile', { name: item.label }), { kind: 'warning' });
            if (ok) await deleteFile(item.key).then(() => opts.onRefresh());
          },
        },
      ];
    }

    default:
      return [];
  }
}
