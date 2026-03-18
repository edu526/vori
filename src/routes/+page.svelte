<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
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
      navigationStore.init(configStore.categories, configStore.projects, configStore.files);
    }
  });

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

<!-- Global overlays — rendered outside the loading guard so dialogs/menus are always available -->
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
