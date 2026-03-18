<script lang="ts">
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { contextMenuStore } from '$lib/stores/contextMenu.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { buildMenuItems } from '../context-menu/menuBuilder';
  import type { NavItem } from '$lib/stores/navigation.svelte';
  import Column from './Column.svelte';

  let isEmpty = $derived(
    Object.keys(configStore.categories).length === 0 &&
    Object.keys(configStore.files).length === 0 &&
    configStore.recents.length === 0,
  );

  async function refresh() {
    await configStore.load();
    navigationStore.refresh(
      configStore.categories,
      configStore.projects,
      configStore.files,
      configStore.favorites,
      configStore.recents,
    );
  }

  function handleRightClick(item: NavItem, x: number, y: number) {
    const menuItems = buildMenuItems(item, {
      defaultEditor: configStore.preferences.default_editor,
      favorites: configStore.favorites,
      onEdit: () => {
        if (item.type === 'category') {
          dialogStore.open({ type: 'category', mode: 'edit', key: item.key });
        } else if (item.type === 'subcategory' && item.categoryKey) {
          dialogStore.open({ type: 'category', mode: 'edit', key: item.key, parentKey: item.categoryKey });
        } else if (item.type === 'project') {
          dialogStore.open({ type: 'project', mode: 'edit', key: item.key });
        } else if (item.type === 'file') {
          dialogStore.open({ type: 'file', mode: 'edit', key: item.key });
        }
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
  {#if isEmpty}
    <div class="onboarding">
      <svg class="onboarding-icon" viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M6 12a3 3 0 0 1 3-3h10l4 5h16a3 3 0 0 1 3 3v20a3 3 0 0 1-3 3H9a3 3 0 0 1-3-3V12z"/>
        <line x1="24" y1="20" x2="24" y2="32"/>
        <line x1="18" y1="26" x2="30" y2="26"/>
      </svg>
      <p class="onboarding-title">Welcome to Vori</p>
      <p class="onboarding-sub">Organize your projects in categories to get started</p>
      <button class="onboarding-btn" onclick={() => dialogStore.open({ type: 'category', mode: 'add' })}>
        Add your first category
      </button>
    </div>
  {:else}
    {#each navigationStore.columns as column, i (i)}
      <Column {column} columnIndex={i} onselect={handleSelect} onrightclick={handleRightClick} />
    {/each}
  {/if}
</div>

<style>
  .column-browser {
    display: flex;
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    background: var(--color-surface);
  }

  .onboarding {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--color-text-secondary);
    padding: 32px;
  }

  .onboarding-icon {
    width: 56px;
    height: 56px;
    opacity: 0.3;
    margin-bottom: 4px;
  }

  .onboarding-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text);
    margin: 0;
  }

  .onboarding-sub {
    font-size: 0.8rem;
    margin: 0;
    text-align: center;
  }

  .onboarding-btn {
    margin-top: 8px;
    padding: 8px 18px;
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .onboarding-btn:hover {
    opacity: 0.85;
  }
</style>
