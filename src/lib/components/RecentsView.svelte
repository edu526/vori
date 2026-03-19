<script lang="ts">
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { openProjectInEditor, openFileInEditor, openInTerminal, addRecent } from '$lib/api/commands';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import ItemIcon from '$lib/components/ItemIcon.svelte';

  const MAX_RECENTS = 8;

  let recents = $derived(configStore.recents.slice(0, MAX_RECENTS));
  let defaultEditor = $derived(configStore.preferences.default_editor);

  const EDITOR_LABELS: Record<string, string> = {
    vscode: 'VSCode', 'vscode-insiders': 'VSCode Insiders', cursor: 'Cursor',
    windsurf: 'Windsurf', kiro: 'Kiro', zed: 'Zed', fleet: 'Fleet',
    sublime: 'Sublime Text', graviton: 'Graviton', helix: 'Helix',
    neovim: 'Neovim', vim: 'Vim', emacs: 'Emacs', kate: 'Kate', gedit: 'Gedit',
  };

  function editorLabel(id: string) {
    return EDITOR_LABELS[id] ?? id;
  }

  function trackRecent(path: string, name: string, type: 'project' | 'file') {
    const item = { path, name, type, timestamp: Date.now() / 1000 };
    addRecent(item);
    navigationStore.addRecentToView(item);
    configStore.recents = [item, ...configStore.recents.filter(r => r.path !== path)].slice(0, 20);
  }

  async function handleOpenProject(path: string, name: string, editor: string) {
    await openProjectInEditor(path, editor);
    trackRecent(path, name, 'project');
    if (configStore.preferences.close_on_open_editor) await getCurrentWindow().close();
  }

  async function handleOpenTerminal(path: string) {
    await openInTerminal(path);
    if (configStore.preferences.close_on_open_terminal) await getCurrentWindow().close();
  }

  async function handleOpenFile(path: string, name: string) {
    await openFileInEditor(path, configStore.preferences.default_text_editor);
    trackRecent(path, name, 'file');
    if (configStore.preferences.close_on_open_file) await getCurrentWindow().close();
  }

  function formatPath(path: string) {
    return path.replace(/^\/home\/[^/]+/, '~');
  }
</script>

<div class="recents-view" role="region" oncontextmenu={(e) => e.preventDefault()}>
  <h2 class="title">Recent</h2>
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
            <button onclick={() => handleOpenProject(item.path, item.name, defaultEditor)}>
              Open in {editorLabel(defaultEditor)}
            </button>
            <button onclick={() => handleOpenTerminal(item.path)}>Terminal</button>
          {:else}
            <button onclick={() => handleOpenFile(item.path, item.name)}>Open</button>
          {/if}
        </div>
      </li>
    {/each}
  </ul>
</div>

<style>
  .recents-view {
    flex: 1;
    padding: 24px 32px;
    overflow-y: auto;
    border-left: 1px solid var(--color-border);
    background: var(--color-bg);
  }

  .title {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin: 0 0 12px;
  }

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
    padding: 8px 10px;
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


  .item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .item-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-path {
    font-size: 11px;
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
    font-size: 11px;
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
