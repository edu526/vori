import { ask } from '@tauri-apps/plugin-dialog';
import type { NavItem } from '$lib/stores/navigation.svelte';
import { navigationStore } from '$lib/stores/navigation.svelte';
import { configStore } from '$lib/stores/config.svelte';
import type { Favorites } from '$lib/api/types';
import type { MenuItem } from '$lib/stores/contextMenu.svelte';
import {
  openProjectInEditor,
  openInTerminal,
  openFileInEditor,
  deleteCategory,
  deleteSubcategory,
  deleteProject,
  deleteFile,
  toggleFavorite,
} from '$lib/api/commands';

export function buildMenuItems(
  item: NavItem,
  opts: {
    defaultEditor: string;
    favorites: Favorites;
    onEdit: () => void;
    onRefresh: () => void;
    onAddSubcategory?: () => void;
    onAddProject?: () => void;
  },
): MenuItem[] {
  const primaryEditor = opts.defaultEditor === 'kiro' ? 'Kiro' : 'VSCode';
  const secondaryEditor = opts.defaultEditor === 'kiro' ? 'VSCode' : 'Kiro';
  const secondaryEditorId = opts.defaultEditor === 'kiro' ? 'vscode' : 'kiro';

  switch (item.type) {
    case 'category': {
      return [
        {
          label: 'Add Subcategory',
          action: () => opts.onAddSubcategory?.(),
        },
        {
          label: 'Add Project here',
          action: () => opts.onAddProject?.(),
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

    case 'subcategory': {
      return [
        {
          label: 'Add Project here',
          action: () => opts.onAddProject?.(),
        },
        { label: '', action: () => {}, divider: true },
        {
          label: 'Edit Subcategory',
          action: () => opts.onEdit(),
        },
        {
          label: 'Delete Subcategory',
          danger: true,
          action: async () => {
            const ok = await ask(`Delete subcategory "${item.label}"? This cannot be undone.`, { kind: 'warning' });
            if (ok) await deleteSubcategory(item.categoryKey!, item.key).then(() => opts.onRefresh());
          },
        },
      ];
    }

    case 'project': {
      const isFav = opts.favorites.projects.includes(item.key);
      return [
        {
          label: `Open in ${primaryEditor}`,
          action: () => openProjectInEditor(item.path!, opts.defaultEditor),
        },
        {
          label: `Open in ${secondaryEditor}`,
          action: () => openProjectInEditor(item.path!, secondaryEditorId),
        },
        {
          label: 'Open in Terminal',
          action: () => openInTerminal(item.path!),
        },
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
