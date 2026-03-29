<script lang="ts">
  import { configStore } from '$lib/stores/config.svelte';
  import { openProjectInEditor, openFileInEditor, openInTerminal, addRecent } from '$lib/api/commands';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import ItemIcon from '$lib/components/ItemIcon.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';

  const MAX_RECENTS = 8;

  const EDITOR_LABELS: Record<string, string> = {
    vscode: 'VSCode', 'vscode-insiders': 'VSCode Insiders', cursor: 'Cursor',
    windsurf: 'Windsurf', kiro: 'Kiro', zed: 'Zed', fleet: 'Fleet',
    sublime: 'Sublime Text', graviton: 'Graviton', helix: 'Helix',
    neovim: 'Neovim', vim: 'Vim', emacs: 'Emacs', kate: 'Kate', gedit: 'Gedit',
  };

  function editorLabel(id: string) {
    return EDITOR_LABELS[id] ?? id;
  }

  // ── Derived state ────────────────────────────────────────────────────────────

  const defaultEditor = $derived(configStore.preferences.default_editor);
  const recents = $derived(configStore.recents.slice(0, MAX_RECENTS));

  const favItems = $derived(() => {
    const items: { key: string; name: string; path: string; type: 'project' | 'file' }[] = [];
    for (const key of configStore.favorites.projects) {
      const p = configStore.projects[key];
      if (p) items.push({ key, name: key, path: p.path, type: 'project' });
    }
    for (const key of configStore.favorites.files) {
      const f = configStore.files[key];
      if (f) items.push({ key, name: key, path: f.path, type: 'file' });
    }
    return items;
  });

  const hasFavorites = $derived(favItems().length > 0);
  const hasRecents = $derived(recents.length > 0);
  const isEmpty = $derived(!hasFavorites && !hasRecents);

  // ── Actions ──────────────────────────────────────────────────────────────────

  function formatPath(path: string) {
    return path.replace(/^\/home\/[^/]+/, '~');
  }

  function trackRecent(path: string, name: string, type: 'project' | 'file') {
    const item = { path, name, type, timestamp: Date.now() / 1000 };
    addRecent(item);
    navigationStore.addRecentToView(item);
    configStore.recents = [item, ...configStore.recents.filter(r => r.path !== path)].slice(0, 20);
  }

  async function openProject(path: string, name: string) {
    await openProjectInEditor(path, defaultEditor);
    trackRecent(path, name, 'project');
    if (configStore.preferences.close_on_open_editor) await getCurrentWindow().close();
  }

  async function openTerminal(path: string) {
    await openInTerminal(path);
    if (configStore.preferences.close_on_open_terminal) await getCurrentWindow().close();
  }

  async function openFile(path: string, name: string) {
    await openFileInEditor(path, configStore.preferences.default_text_editor);
    trackRecent(path, name, 'file');
    if (configStore.preferences.close_on_open_file) await getCurrentWindow().close();
  }
</script>

<div class="home-view" role="region" oncontextmenu={(e) => e.preventDefault()}>

  {#if isEmpty}
    <div class="empty-state">
      <span class="empty-icon">⌘</span>
      <p class="empty-title">No recent projects</p>
      <p class="empty-hint">Open a category to browse your projects</p>
    </div>

  {:else}

    {#if hasFavorites}
      <section>
        <h2 class="section-title">Favorites</h2>
        <ul class="list">
          {#each favItems() as item (item.key)}
            <li class="item">
              <div class="item-icon starred">
                <ItemIcon type={item.type} size={16} />
              </div>
              <div class="item-info">
                <span class="item-name">{item.name}</span>
                <span class="item-path">{formatPath(item.path)}</span>
              </div>
              <div class="item-actions">
                {#if item.type === 'project'}
                  <button onclick={() => openProject(item.path, item.name)}>
                    Open in {editorLabel(defaultEditor)}
                  </button>
                  <button onclick={() => openTerminal(item.path)}>Terminal</button>
                {:else}
                  <button onclick={() => openFile(item.path, item.name)}>Open</button>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {#if hasFavorites && hasRecents}
      <div class="divider"></div>
    {/if}

    {#if hasRecents}
      <section>
        <h2 class="section-title">Recent</h2>
        <ul class="list">
          {#each recents as item (item.path)}
            <li class="item">
              <div class="item-icon">
                <ItemIcon type={item.type === 'file' ? 'file' : 'project'} size={16} />
              </div>
              <div class="item-info">
                <span class="item-name">{item.name}</span>
                <span class="item-path">{formatPath(item.path)}</span>
              </div>
              <div class="item-actions">
                {#if item.type === 'project'}
                  <button onclick={() => openProject(item.path, item.name)}>
                    Open in {editorLabel(defaultEditor)}
                  </button>
                  <button onclick={() => openTerminal(item.path)}>Terminal</button>
                {:else}
                  <button onclick={() => openFile(item.path, item.name)}>Open</button>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

  {/if}
</div>

<style>
  .home-view {
    flex: 1;
    padding: 20px 28px;
    overflow-y: auto;
    border-left: 1px solid var(--color-border);
    background: var(--color-bg);
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  /* ── Empty state ── */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    color: var(--color-text-secondary);
  }

  .empty-icon {
    font-size: var(--text-2xl);
    opacity: 0.3;
    margin-bottom: 4px;
  }

  .empty-title {
    font-size: var(--text-md);
    font-weight: 500;
    margin: 0;
    color: var(--color-text-secondary);
  }

  .empty-hint {
    font-size: var(--text-xs);
    margin: 0;
    opacity: 0.7;
  }

  /* ── Sections ── */
  section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-title {
    font-size: var(--text-xs);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin: 0 0 6px;
    padding: 0 10px;
  }

  .divider {
    height: 1px;
    background: var(--color-border);
    margin: 14px 10px;
  }

  /* ── Items ── */
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border-radius: 7px;
    cursor: default;
  }

  .item:hover {
    background: var(--color-hover);
  }

  .item-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    color: var(--color-text-secondary);
  }

  .item-icon.starred {
    color: var(--color-accent);
  }

  .item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .item-name {
    font-size: var(--text-base);
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-path {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.1s;
  }

  .item:hover .item-actions {
    opacity: 1;
  }

  .item-actions button {
    font-size: var(--text-xs);
    padding: 3px 8px;
    border-radius: 4px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    cursor: pointer;
    white-space: nowrap;
  }

  .item-actions button:hover {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
  }

  .item-actions button:first-child {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
  }

  .item-actions button:first-child:hover {
    opacity: 0.85;
  }
</style>
