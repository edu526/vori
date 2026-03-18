<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { contextMenuStore } from '$lib/stores/contextMenu.svelte';
  import { openProjectInEditor, openFileInEditor, addRecent } from '$lib/api/commands';
  import ColumnBrowser from '$lib/components/columns/ColumnBrowser.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import DetailPanel from '$lib/components/DetailPanel.svelte';
  import ContextMenu from '$lib/components/context-menu/ContextMenu.svelte';
  import CategoryDialog from '$lib/components/dialogs/CategoryDialog.svelte';
  import ProjectDialog from '$lib/components/dialogs/ProjectDialog.svelte';
  import PreferencesDialog from '$lib/components/dialogs/PreferencesDialog.svelte';
  import type { SearchResult } from '$lib/api/types';

  onMount(async () => {
    await themeStore.init();
    await configStore.load();
    if (!configStore.error) {
      navigationStore.init(
        configStore.categories,
        configStore.projects,
        configStore.files,
        configStore.favorites,
        configStore.recents,
      );
    }
  });

  // ── Keyboard navigation ─────────────────────────────────────────────────────
  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      // Let dialogs and context menus handle their own keys
      if (dialogStore.current) return;
      if (contextMenuStore.visible) return;
      // Don't intercept when typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          navigationStore.moveSelection(-1);
          break;
        case 'ArrowDown':
          e.preventDefault();
          navigationStore.moveSelection(1);
          break;
        case 'ArrowRight':
          e.preventDefault();
          navigationStore.expandRight();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          navigationStore.collapseLeft();
          break;
        case 'Enter': {
          e.preventDefault();
          const item = navigationStore.selectedItem;
          if (item?.type === 'project' && item.path) {
            openProjectInEditor(item.path, configStore.preferences.default_editor).then(() =>
              addRecent({
                path: item.path!,
                name: item.label,
                type: 'project',
                timestamp: Date.now() / 1000,
              }),
            );
          } else if (item?.type === 'file' && item.path) {
            openFileInEditor(item.path, configStore.preferences.default_text_editor).then(() =>
              addRecent({
                path: item.path!,
                name: item.label,
                type: 'file',
                timestamp: Date.now() / 1000,
              }),
            );
          }
          break;
        }
      }
    }

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  // ── Search navigation ───────────────────────────────────────────────────────
  function handleSearchResult(result: SearchResult) {
    if (result.result_type === 'category') {
      navigationStore.selectItem(0, result.key);
    } else if (result.result_type === 'subcategory' && result.category) {
      navigationStore.selectItem(0, result.category);
      setTimeout(() => navigationStore.selectItem(1, result.subcategory!), 50);
    } else if (result.result_type === 'project' && result.category) {
      navigationStore.selectItem(0, result.category);
      setTimeout(() => {
        if (result.subcategory) {
          navigationStore.selectItem(1, result.subcategory);
          setTimeout(() => navigationStore.selectItem(2, result.key), 50);
        } else {
          navigationStore.selectItem(1, result.key);
        }
      }, 50);
    }
  }
</script>

<div class="app-shell">
  {#if configStore.loading}
    <div class="state-overlay">Loading...</div>
  {:else if configStore.error}
    <div class="state-overlay error">Failed to load config: {configStore.error}</div>
  {:else}
    <Toolbar
      onnewcategory={() => dialogStore.open({ type: 'category', mode: 'add' })}
      onnewproject={() => dialogStore.open({ type: 'project', mode: 'add' })}
      onopenpreferences={() => dialogStore.open({ type: 'preferences' })}
      onsearchresult={handleSearchResult}
    />
    <div class="main-content">
      <ColumnBrowser />
      <DetailPanel />
    </div>
  {/if}
</div>

<!-- Global overlays -->
<ContextMenu />
{#if dialogStore.current?.type === 'category'}
  <CategoryDialog />
{:else if dialogStore.current?.type === 'project'}
  <ProjectDialog />
{:else if dialogStore.current?.type === 'preferences'}
  <PreferencesDialog />
{/if}

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .state-overlay {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    color: var(--color-text-secondary);
    font-size: 0.9rem;
  }

  .state-overlay.error {
    color: #c0392b;
  }
</style>
