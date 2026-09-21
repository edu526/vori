<script lang="ts">
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { contextMenuStore } from '$lib/stores/contextMenu.svelte';
  import { openProjectInEditor, openFileInEditor, addRecent, updatePreferences, takePendingRequests } from '$lib/api/commands';
  import { isTextFile } from '$lib/utils/textExtensions';
  import { openEditor } from '$lib/utils/openEditor';

  import ColumnBrowser from '$lib/components/columns/ColumnBrowser.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';

  import ContextMenu from '$lib/components/context-menu/ContextMenu.svelte';
  import CategoryDialog from '$lib/components/dialogs/CategoryDialog.svelte';
  import ProjectDialog from '$lib/components/dialogs/ProjectDialog.svelte';
  import FileDialog from '$lib/components/dialogs/FileDialog.svelte';
  import PreferencesDialog from '$lib/components/dialogs/PreferencesDialog.svelte';
  import ImportFolderModal from '$lib/components/dialogs/ImportFolderModal.svelte';
  import CloneDialog from '$lib/components/dialogs/CloneDialog.svelte';
  import HomeView from '$lib/components/HomeView.svelte';
  import WorkspaceBar from '$lib/components/WorkspaceBar.svelte';
  import SearchModal from '$lib/components/SearchModal.svelte';
  import type { SearchResult } from '$lib/api/types';
  import { message } from '@tauri-apps/plugin-dialog';
  import { syncStore } from '$lib/stores/sync.svelte';
  import { listen } from '@tauri-apps/api/event';
  import { t, i18n } from '$lib/i18n/index.svelte';

  const isEditorOpen = $derived(dialogStore.current?.type === 'editor');

  let searchModalOpen = $state(false);

  onMount(async () => {
    await configStore.load();
    themeStore.apply(configStore.preferences.theme ?? 'system');
    i18n.setPreference(configStore.preferences.language ?? 'system');
    themeStore.applyScale(configStore.preferences.ui_scale ?? 1.0);
    if (!configStore.error) {
      navigationStore.init(
        configStore.categories,
        configStore.projects,
        configStore.files,
        configStore.favorites,
        configStore.recents,
      );
      await navigationStore.loadWorkspaceSelection();
      syncStore.start();
      await drainCliRequests();
      if (configStore.recoveryNotes.length > 0) {
        const text = configStore.recoveryNotes
          .map((n) =>
            n.backup
              ? t('recovery.resetKept', { file: n.file, backup: n.backup })
              : t('recovery.reset', { file: n.file }),
          )
          .join('\n\n');
        await message(text, {
          title: t('recovery.title'),
          kind: 'warning',
        });
      }
    }
  });

  // ── Requests from the command line / vori:// links ──────────────────────────
  async function drainCliRequests() {
    if (configStore.loading || configStore.error) return; // the post-load drain picks them up
    for (const request of await takePendingRequests()) {
      if (request.kind === 'add-or-reveal') {
        const existing = request.existing ? configStore.projects[request.existing] : undefined;
        if (request.existing && existing) {
          handleSearchResult({
            key: request.existing,
            name: request.existing,
            result_type: 'project',
            path: existing.path,
            parent: existing.parent,
          });
        } else {
          const name = request.path.split(/[\\/]/).filter(Boolean).pop() ?? '';
          dialogStore.open({ type: 'project', mode: 'add', prefill: { name, path: request.path } });
        }
      } else if (request.kind === 'opened') {
        const recent = { path: request.path, name: request.name, type: 'project' as const, timestamp: Date.now() / 1000 };
        navigationStore.addRecentToView(recent);
        configStore.recents = [recent, ...configStore.recents.filter((r) => r.path !== recent.path)].slice(0, 20);
      } else {
        const text =
          request.kind === 'no-project'
            ? t('cli.noProject', { query: request.query })
            : request.kind === 'ambiguous-project'
              ? t('cli.ambiguous', { query: request.query, names: request.matches.join(', ') })
              : request.message;
        await message(text, { title: 'Vori', kind: 'warning' });
      }
    }
  }

  $effect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    listen('cli-request', () => void drainCliRequests()).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  // ── Block native context menu globally ──────────────────────────────────────
  $effect(() => {
    const block = (e: MouseEvent) => e.preventDefault();
    window.addEventListener('contextmenu', block);
    return () => window.removeEventListener('contextmenu', block);
  });

  // ── Editor close → deselect deepest column ──────────────────────────────────
  // ponytail: the file column was selected for the editor; once it's gone,
  // collapse the deepest selection so HomeView (recents) reappears. But only
  // when no other navigation happened (back button, breadcrumb, category
  // click) — those paths already reshape the columns themselves.
  let prevEditorOpen = false;
  let prevSelectedKey: string | null = null;
  $effect(() => {
    const editorOpen = dialogStore.current?.type === 'editor';
    const selectedKey = navigationStore.selectedItem?.key ?? null;
    if (prevEditorOpen && !editorOpen && selectedKey === prevSelectedKey) {
      navigationStore.collapseDeepest();
    }
    prevEditorOpen = editorOpen;
    prevSelectedKey = selectedKey;
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
    async function handleKeydown(e: KeyboardEvent) {
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
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const { LogicalSize } = await import('@tauri-apps/api/dpi');
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
            if (isTextFile(item.path)) {
              await openEditor(item.path, item.label);
              return;
            }
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
    <div class="state-overlay">{t('app.loading')}</div>
  {:else if configStore.error}
    <div class="state-overlay error">{t('app.loadFailed', { error: configStore.error ?? '' })}</div>
  {:else}
    <Toolbar
      onopenpreferences={() => dialogStore.open({ type: 'preferences' })}
      onsearchopen={() => { searchModalOpen = true; }}
    />
    <div class="main-content">
      <ColumnBrowser />
      {#if !isEditorOpen && navigationStore.columns[0]?.selectedKey === null}
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
{:else if dialogStore.current?.type === 'clone'}
  <CloneDialog />
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
