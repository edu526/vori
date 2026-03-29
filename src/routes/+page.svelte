<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { contextMenuStore } from '$lib/stores/contextMenu.svelte';
  import { openProjectInEditor, openFileInEditor, addRecent, updatePreferences } from '$lib/api/commands';
  import { LogicalSize } from '@tauri-apps/api/dpi';
  import ColumnBrowser from '$lib/components/columns/ColumnBrowser.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';

  import ContextMenu from '$lib/components/context-menu/ContextMenu.svelte';
  import CategoryDialog from '$lib/components/dialogs/CategoryDialog.svelte';
  import ProjectDialog from '$lib/components/dialogs/ProjectDialog.svelte';
  import FileDialog from '$lib/components/dialogs/FileDialog.svelte';
  import PreferencesDialog from '$lib/components/dialogs/PreferencesDialog.svelte';
  import ImportFolderModal from '$lib/components/dialogs/ImportFolderModal.svelte';
  import HomeView from '$lib/components/HomeView.svelte';
  import WorkspaceBar from '$lib/components/WorkspaceBar.svelte';
  import SearchModal from '$lib/components/SearchModal.svelte';
  import type { SearchResult } from '$lib/api/types';

  let searchModalOpen = $state(false);

  onMount(async () => {
    await configStore.load();
    themeStore.apply(configStore.preferences.theme ?? 'system');
    themeStore.applyScale(configStore.preferences.ui_scale ?? 1.0);
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

  // ── Block native context menu globally ──────────────────────────────────────
  $effect(() => {
    const block = (e: MouseEvent) => e.preventDefault();
    window.addEventListener('contextmenu', block);
    return () => window.removeEventListener('contextmenu', block);
  });

  // ── Scale persistence helper ────────────────────────────────────────────────
  async function persistScale(scale: number) {
    const updated = { ...configStore.preferences, ui_scale: scale };
    configStore.preferences = updated;
    await updatePreferences(updated);
  }

  // Default window dimensions (matches tauri.conf.json)
  const DEFAULT_W = 960;
  const DEFAULT_H = 640;

  // ── Keyboard navigation ─────────────────────────────────────────────────────
  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        if (dialogStore.handleEscape()) return;
        if (contextMenuStore.visible) { contextMenuStore.hide(); return; }
        navigationStore.collapseDeepest();
        return;
      }
      if (dialogStore.current) return;
      if (contextMenuStore.visible) return;
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        e.preventDefault();
        searchModalOpen = true;
        return;
      }
      if ((e.ctrlKey || e.metaKey) && (e.key === '=' || e.key === '+')) {
        e.preventDefault();
        persistScale(themeStore.step(0.1, configStore.preferences.ui_scale ?? 1.0));
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '-') {
        e.preventDefault();
        persistScale(themeStore.step(-0.1, configStore.preferences.ui_scale ?? 1.0));
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '0') {
        e.preventDefault();
        themeStore.applyScale(1.0);
        persistScale(1.0);
        getCurrentWindow().setSize(new LogicalSize(DEFAULT_W, DEFAULT_H));
        return;
      }

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
            const recent = { path: item.path, name: item.label, type: 'project' as const, timestamp: Date.now() / 1000 };
            openProjectInEditor(item.path, configStore.preferences.default_editor).then(() => {
              addRecent(recent);
              navigationStore.addRecentToView(recent);
              configStore.recents = [recent, ...configStore.recents.filter(r => r.path !== recent.path)].slice(0, 20);
            });
          } else if (item?.type === 'file' && item.path) {
            const recent = { path: item.path, name: item.label, type: 'file' as const, timestamp: Date.now() / 1000 };
            openFileInEditor(item.path, configStore.preferences.default_text_editor).then(() => {
              addRecent(recent);
              navigationStore.addRecentToView(recent);
              configStore.recents = [recent, ...configStore.recents.filter(r => r.path !== recent.path)].slice(0, 20);
            });
          }
          break;
        }
      }
    }

    window.addEventListener('keydown', handleKeydown, true);
    return () => window.removeEventListener('keydown', handleKeydown, true);
  });

  // ── Search navigation ───────────────────────────────────────────────────────
  function buildAncestorChain(categoryKey: string): string[] {
    const chain: string[] = [];
    let current: string | null = categoryKey;
    while (current) {
      chain.unshift(current);
      current = configStore.categories[current]?.parent ?? null;
    }
    return chain;
  }

  function handleSearchResult(result: SearchResult) {
    if (result.result_type === 'category') {
      const chain = buildAncestorChain(result.key);
      // Select each ancestor in sequence, then the category itself
      let delay = 0;
      chain.forEach((key, i) => {
        setTimeout(() => navigationStore.selectItem(i, key), delay);
        delay += 50;
      });
    } else if (result.result_type === 'project' && result.parent) {
      const chain = buildAncestorChain(result.parent);
      let delay = 0;
      chain.forEach((key, i) => {
        setTimeout(() => navigationStore.selectItem(i, key), delay);
        delay += 50;
      });
      setTimeout(() => navigationStore.selectItem(chain.length, result.key), delay);
    } else if (result.result_type === 'file') {
      // Files are always at root column
      navigationStore.selectItem(0, result.key);
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
      onopenpreferences={() => dialogStore.open({ type: 'preferences' })}
      onsearchopen={() => { searchModalOpen = true; }}
    />
    <div class="main-content">
      <ColumnBrowser />
      {#if navigationStore.columns[0]?.selectedKey === null}
        <HomeView />
      {/if}
    </div>
  {/if}
</div>

<!-- Global overlays -->
<WorkspaceBar />
<ContextMenu />
{#if searchModalOpen}
  <SearchModal
    onclose={() => { searchModalOpen = false; }}
    onresult={(result) => { searchModalOpen = false; handleSearchResult(result); }}
  />
{/if}
{#if dialogStore.current?.type === 'category'}
  <CategoryDialog />
{:else if dialogStore.current?.type === 'project'}
  <ProjectDialog />
{:else if dialogStore.current?.type === 'file'}
  <FileDialog />
{:else if dialogStore.current?.type === 'preferences'}
  <PreferencesDialog />
{:else if dialogStore.current?.type === 'import-folder'}
  <ImportFolderModal />
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
    font-size: var(--text-md);
  }

  .state-overlay.error {
    color: #c0392b;
  }
</style>
