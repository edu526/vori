<script lang="ts">
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { contextMenuStore } from '$lib/stores/contextMenu.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { buildMenuItems } from '../context-menu/menuBuilder';
  import type { NavItem } from '$lib/stores/navigation.svelte';
  import Column from './Column.svelte';

  async function refresh() {
    await configStore.load();
    navigationStore.refresh(configStore.categories, configStore.projects, configStore.files);
  }

  function handleRightClick(item: NavItem, x: number, y: number) {
    const menuItems = buildMenuItems(item, {
      defaultEditor: configStore.preferences.default_editor,
      favorites: configStore.favorites,
      onEdit: () => {
        if (item.type === 'category') {
          dialogStore.open({ type: 'category', mode: 'edit', key: item.key });
        } else if (item.type === 'project') {
          dialogStore.open({ type: 'project', mode: 'edit', key: item.key });
        }
        // subcategory & file edit: future dialogs
      },
      onRefresh: refresh,
      onAddSubcategory: () => {
        dialogStore.open({ type: 'category', mode: 'add', parentKey: item.key });
      },
      onAddProject: () => {
        dialogStore.open({
          type: 'project',
          mode: 'add',
          categoryKey: item.categoryKey ?? item.key,
          subcategoryKey: item.type === 'subcategory' ? item.key : undefined,
        });
      },
    });
    contextMenuStore.show(x, y, menuItems);
  }

  function handleSelect(columnIndex: number, key: string) {
    navigationStore.selectItem(columnIndex, key);
  }
</script>

<div class="column-browser">
  {#each navigationStore.columns as column, i (i)}
    <Column {column} columnIndex={i} onselect={handleSelect} onrightclick={handleRightClick} />
  {/each}
</div>

<style>
  .column-browser {
    display: flex;
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    background: var(--color-surface);
  }
</style>
