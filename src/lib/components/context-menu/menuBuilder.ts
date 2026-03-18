import type { NavItem } from '$lib/stores/navigation.svelte';
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
          action: () => {
            if (window.confirm(`Delete category "${item.label}"? This cannot be undone.`)) {
              deleteCategory(item.key).then(() => opts.onRefresh());
            }
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
          action: () => {
            if (
              window.confirm(`Delete subcategory "${item.label}"? This cannot be undone.`)
            ) {
              deleteSubcategory(item.categoryKey!, item.key).then(() => opts.onRefresh());
            }
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
            toggleFavorite(item.key, 'project').then(() => opts.onRefresh());
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
          action: () => {
            if (window.confirm(`Delete project "${item.label}"? This cannot be undone.`)) {
              deleteProject(item.key).then(() => opts.onRefresh());
            }
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
            toggleFavorite(item.key, 'file').then(() => opts.onRefresh());
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
          action: () => {
            if (window.confirm(`Delete file "${item.label}"? This cannot be undone.`)) {
              deleteFile(item.key).then(() => opts.onRefresh());
            }
          },
        },
      ];
    }

    default:
      return [];
  }
}
