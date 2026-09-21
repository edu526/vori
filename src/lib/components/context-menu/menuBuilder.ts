import { ask, message, open } from '@tauri-apps/plugin-dialog';
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
          ? `Your terminal can't run commands from Vori, so it was opened in the project folder.\n\n"${result.command}" is on your clipboard: paste it there.`
          : `Your terminal can't run commands from Vori. Run this in the project folder:\n\n${result.command}`,
        { title: 'Run script', kind: 'info' },
      );
    }
  } catch (e) {
    await message(String(e), { title: 'Could not run script', kind: 'error' });
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
          label: 'Add Subcategory',
          action: () => opts.onAddChildCategory?.(),
        },
        {
          label: 'Add Project here',
          action: () => opts.onAddProject?.(),
        },
        {
          label: 'Clone repository…',
          action: () => opts.onCloneRepo?.(),
        },
        {
          label: 'Import folder…',
          action: () => opts.onImportFolder?.(),
        },
        ...(newFolders > 0
          ? [
              {
                label: `Import ${newFolders} new project${newFolders !== 1 ? 's' : ''}…`,
                action: () => opts.onImportFolder?.(existingSourcePath ?? undefined),
              },
              {
                label: 'Ignore new folders',
                action: () => syncStore.dismiss(item.key),
              },
            ]
          : []),
        ...(existingSourcePath
          ? [
              {
                label: 'Refresh Import Tree…',
                action: () => opts.onImportFolder?.(existingSourcePath),
              },
            ]
          : []),
        { label: '', action: () => {}, divider: true },
        {
          label: existingSourcePath ? 'Detect Workspaces' : 'Detect Workspaces in folder…',
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
                label: `Import ${detected.length} workspace${detected.length !== 1 ? 's' : ''} as project${detected.length !== 1 ? 's' : ''}`,
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
          label: 'Edit Category',
          action: () => opts.onEdit(),
        },
        {
          label: 'Delete Category',
          danger: true,
          action: async () => {
            const ok = await ask(`Delete category "${item.label}"? This cannot be undone.`, { kind: 'warning' });
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
          label: `Open in ${primaryLabel}`,
          action: () => openProjectInEditor(item.path!, opts.defaultEditor),
        },
        ...otherEditors.map((key) => ({
          label: `Open in ${editorLabel(key)}`,
          action: () => openProjectInEditor(item.path!, key),
        })),
        {
          label: 'Open in Terminal',
          action: () => openInTerminal(item.path!),
        },
        ...(opts.scripts && opts.scripts.scripts.length > 0
          ? [
              { label: '', action: () => {}, divider: true },
              ...opts.scripts.scripts.slice(0, MAX_MENU_SCRIPTS).map((script) => ({
                label: `Run: ${script}`,
                action: () => runScript(item.path!, script),
              })),
            ]
          : []),
        { label: '', action: () => {}, divider: true },
        {
          label: 'Show in File Manager',
          action: () => revealItemInDir(item.path!),
        },
        {
          label: 'Copy Path',
          action: () => { void copyText(item.path!); },
        },
        { label: '', action: () => {}, divider: true },
        {
          label: inWorkspace ? 'Remove from workspace selection' : 'Add to workspace selection',
          action: () => navigationStore.toggleWorkspaceItem(item.key, item.path!, item.label),
        },
        ...(workspaceSize >= 2 && inWorkspace || workspaceSize >= 1 && !inWorkspace
          ? [{
              label: `Open ${workspaceSize + (inWorkspace ? 0 : 1)} projects as workspace`,
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
          label: isFav ? 'Remove from Favorites' : 'Add to Favorites',
          action: () => {
            toggleFavorite(item.key, 'project').then((favs) => { navigationStore.updateFavorites(favs); configStore.favorites = favs; });
          },
        },
        { label: '', action: () => {}, divider: true },
        {
          label: 'Edit Project',
          action: () => opts.onEdit(),
        },
        {
          label: 'Delete Project',
          danger: true,
          action: async () => {
            const ok = await ask(`Delete project "${item.label}"? This cannot be undone.`, { kind: 'warning' });
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
          label: `Open in ${primaryLabel}`,
          action: () => openProjectInEditor(item.path!, opts.defaultEditor),
        },
        ...otherEditors.map((key) => ({
          label: `Open in ${editorLabel(key)}`,
          action: () => openProjectInEditor(item.path!, key),
        })),
        { label: '', action: () => {}, divider: true },
        {
          label: inWorkspace ? 'Remove from workspace selection' : 'Add to workspace selection',
          action: () => navigationStore.toggleWorkspaceItem(item.key, item.path!, item.label),
        },
        ...(workspaceSize >= 2 && inWorkspace || workspaceSize >= 1 && !inWorkspace
          ? [{
              label: `Open ${workspaceSize + (inWorkspace ? 0 : 1)} as workspace`,
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
          label: 'Import as project',
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
          label: isFav ? 'Remove from Favorites' : 'Add to Favorites',
          action: () => {
            toggleFavorite(item.key, 'file').then((favs) => { navigationStore.updateFavorites(favs); configStore.favorites = favs; });
          },
        },
        { label: '', action: () => {}, divider: true },
        {
          label: 'Edit File',
          action: () => opts.onEdit(),
        },
        {
          label: 'Delete File',
          danger: true,
          action: async () => {
            const ok = await ask(`Delete file "${item.label}"? This cannot be undone.`, { kind: 'warning' });
            if (ok) await deleteFile(item.key).then(() => opts.onRefresh());
          },
        },
      ];
    }

    default:
      return [];
  }
}
