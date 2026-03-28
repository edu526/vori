import { ask } from '@tauri-apps/plugin-dialog';
import type { NavItem } from '$lib/stores/navigation.svelte';
import { navigationStore } from '$lib/stores/navigation.svelte';
import { configStore } from '$lib/stores/config.svelte';
import { openWorkspaceInEditor } from '$lib/api/commands';
import type { Favorites } from '$lib/api/types';
import type { MenuItem } from '$lib/stores/contextMenu.svelte';
import {
  openProjectInEditor,
  openInTerminal,
  openFileInEditor,
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
    onImportFolder?: () => void;
  },
): MenuItem[] {
  const primaryLabel = editorLabel(opts.defaultEditor);
  const otherEditors = Object.keys(opts.editorsAvailable)
    .filter((k) => k !== opts.defaultEditor)
    .sort((a, b) => a.localeCompare(b));

  switch (item.type) {
    case 'category': {
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
          label: 'Import folder…',
          action: () => opts.onImportFolder?.(),
        },
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
      const inWorkspace = navigationStore.workspaceSelection.has(item.key);
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

    case 'file': {
      const isFav = opts.favorites.files.includes(item.key);
      return [
        {
          label: 'Open File',
          action: () => openFileInEditor(item.path!),
        },
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
