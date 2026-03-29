<script lang="ts">
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { contextMenuStore } from '$lib/stores/contextMenu.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { buildMenuItems } from '../context-menu/menuBuilder';
  import { openProjectInEditor, openFileInEditor, openInTerminal, addRecent } from '$lib/api/commands';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { NavItem } from '$lib/stores/navigation.svelte';
  import Column from './Column.svelte';

  const MIN_COL = 180; // minimum column width before sliding kicks in
  const MAX_COL = 280; // maximum column width — avoids stretched columns on wide windows

  let browserWidth = $state(0);

  const columnCount = $derived(navigationStore.columns.length);
  const isSingleCol = $derived(columnCount <= 1);

  // How many columns fit at MIN_COL width
  const maxVisible = $derived(Math.max(1, Math.floor((browserWidth || 600) / MIN_COL)));

  // Each column gets an equal share, clamped between MIN_COL and MAX_COL.
  // Fixed 220px when paired with HomeView (single-column root state).
  const colWidth = $derived(
    isSingleCol
      ? 220
      : Math.min(MAX_COL, (browserWidth || 600) / Math.min(columnCount, maxVisible))
  );

  // Slide offset: keeps the rightmost maxVisible columns in view
  const trackOffset = $derived(
    columnCount > maxVisible ? (columnCount - maxVisible) * colWidth : 0
  );

  const hasHidden = $derived(trackOffset > 0);

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
      editorsAvailable: configStore.preferences.editors_available ?? {},
      favorites: configStore.favorites,
      onEdit: () => {
        if (item.type === 'category') {
          dialogStore.open({ type: 'category', mode: 'edit', key: item.key });
        } else if (item.type === 'project') {
          dialogStore.open({ type: 'project', mode: 'edit', key: item.key });
        } else if (item.type === 'file') {
          dialogStore.open({ type: 'file', mode: 'edit', key: item.key });
        }
      },
      onRefresh: refresh,
      onAddChildCategory: () => {
        dialogStore.open({ type: 'category', mode: 'add', parentKey: item.key });
      },
      onAddProject: () => {
        dialogStore.open({ type: 'project', mode: 'add', parentKey: item.key });
      },
      onImportFolder: () => {
        dialogStore.open({ type: 'import-folder', defaultParent: item.key });
      },
    });
    contextMenuStore.show(x, y, menuItems);
  }

  function handleSelect(columnIndex: number, key: string) {
    navigationStore.selectItem(columnIndex, key);
  }

  async function handleOpen(item: NavItem) {
    if (!item.path) return;
    if (item.type === 'project') {
      const recent = { path: item.path, name: item.label, type: 'project' as const, timestamp: Date.now() / 1000 };
      await openProjectInEditor(item.path, configStore.preferences.default_editor);
      addRecent(recent);
      navigationStore.addRecentToView(recent);
      configStore.recents = [recent, ...configStore.recents.filter(r => r.path !== item.path)].slice(0, 20);
      if (configStore.preferences.close_on_open_editor) await getCurrentWindow().close();
    } else if (item.type === 'file') {
      const recent = { path: item.path, name: item.label, type: 'file' as const, timestamp: Date.now() / 1000 };
      await openFileInEditor(item.path, configStore.preferences.default_text_editor);
      addRecent(recent);
      navigationStore.addRecentToView(recent);
      configStore.recents = [recent, ...configStore.recents.filter(r => r.path !== item.path)].slice(0, 20);
      if (configStore.preferences.close_on_open_file) await getCurrentWindow().close();
    }
  }

  function handleEmptyRightClick(columnIndex: number, x: number, y: number) {
    const col = navigationStore.columns[columnIndex];

    // Root column → New Category / Project / File / Import
    if (columnIndex === 0) {
      contextMenuStore.show(x, y, [
        { label: 'New Category',    action: () => dialogStore.open({ type: 'category',      mode: 'add' }) },
        { label: 'New Project',     action: () => dialogStore.open({ type: 'project',        mode: 'add' }) },
        { label: 'New File',        action: () => dialogStore.open({ type: 'file',           mode: 'add' }) },
        { label: '', action: () => {}, divider: true },
        { label: 'Import folder…',  action: () => dialogStore.open({ type: 'import-folder' }) },
      ]);
      return;
    }

    // Category column → Add Subcategory / Add Project
    const selectedInPrev = navigationStore.columns[columnIndex - 1]?.selectedKey;
    if (selectedInPrev && col?.title) {
      const prevItem = navigationStore.columns[columnIndex - 1].items.find(
        (it) => it.key === selectedInPrev,
      );
      if (prevItem?.type === 'category') {
        contextMenuStore.show(x, y, [
          { label: 'Add Subcategory',  action: () => dialogStore.open({ type: 'category',     mode: 'add', parentKey: selectedInPrev }) },
          { label: 'Add Project here', action: () => dialogStore.open({ type: 'project',       mode: 'add', parentKey: selectedInPrev }) },
          { label: '', action: () => {}, divider: true },
          { label: 'Import folder…',   action: () => dialogStore.open({ type: 'import-folder', defaultParent: selectedInPrev }) },
        ]);
      }
    }
  }
</script>

<div
  class="column-browser"
  class:multi={!isSingleCol}
  class:has-hidden={hasHidden}
  bind:clientWidth={browserWidth}
>
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
    <div class="column-track" style="transform: translateX(-{trackOffset}px)">
      {#each navigationStore.columns as column, i (i)}
        <Column
          {column}
          columnIndex={i}
          active={i === navigationStore.activeColumnIndex}
          width={colWidth}
          onselect={handleSelect}
          onrightclick={handleRightClick}
          onemptyrightclick={handleEmptyRightClick}
          onopen={handleOpen}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .column-browser {
    position: relative;
    display: flex;
    flex: 0 0 auto;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* Fills remaining space when navigating (HomeView is gone) */
  .column-browser.multi {
    flex: 1;
    min-width: 0;
  }

  /* Inner track — slides left via transform, never scrolls */
  .column-track {
    display: flex;
    height: 100%;
    transition: transform 0.22s cubic-bezier(0.4, 0, 0.2, 1);
    will-change: transform;
  }

  /* Left-edge fade when columns are hidden behind the left boundary */
  .column-browser::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 18px;
    background: linear-gradient(to right, var(--color-surface), transparent);
    pointer-events: none;
    z-index: 2;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .column-browser.has-hidden::before {
    opacity: 1;
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
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-text);
    margin: 0;
  }

  .onboarding-sub {
    font-size: var(--text-sm);
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
    font-size: var(--text-base);
    font-weight: 500;
    cursor: pointer;
  }

  .onboarding-btn:hover {
    opacity: 0.85;
  }
</style>
