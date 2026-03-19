<script lang="ts">
  import { getInstalledApps } from '$lib/api/commands';
  import { open } from '@tauri-apps/plugin-dialog';

  type InstalledApp = { name: string; exec: string };

  let {
    onAdd,
    onClose,
    title = 'Add Editor',
  }: {
    onAdd: (name: string, exec: string) => void;
    onClose: () => void;
    title?: string;
  } = $props();

  let mode = $state<'search' | 'browse'>('search');
  let query = $state('');
  let apps = $state<InstalledApp[]>([]);
  let loading = $state(false);
  let browseName = $state('');
  let browsePath = $state('');

  async function loadApps() {
    loading = true;
    try {
      apps = await getInstalledApps();
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (mode === 'search' && apps.length === 0) {
      loadApps();
    }
  });

  const filtered = $derived(
    query.trim()
      ? apps.filter((a) => a.name.toLowerCase().includes(query.toLowerCase()))
      : apps
  );

  function selectApp(app: InstalledApp) {
    onAdd(app.name, app.exec);
  }

  async function handleBrowse() {
    const selected = await open({
      multiple: false,
      title: 'Select editor binary',
    });
    if (!selected) return;
    const path = typeof selected === 'string' ? selected : selected[0];
    browsePath = path;
    if (!browseName) {
      // Use filename without extension as default name
      browseName = path.split('/').pop()?.replace(/\.[^.]+$/, '') ?? '';
    }
  }

  function handleBrowseAdd() {
    if (browsePath && browseName) {
      onAdd(browseName, browsePath);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
    e.stopPropagation();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose} onkeydown={handleKeydown}>
  <div class="modal" role="dialog" aria-modal="true" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <span>{title}</span>
      <button class="close-btn" onclick={onClose}>✕</button>
    </div>

    <div class="tabs">
      <button class="tab" class:active={mode === 'search'} onclick={() => (mode = 'search')}>
        Installed Apps
      </button>
      <button class="tab" class:active={mode === 'browse'} onclick={() => (mode = 'browse')}>
        Browse Binary
      </button>
    </div>

    {#if mode === 'search'}
      <input
        class="search-input"
        type="text"
        placeholder="Search apps..."
        bind:value={query}
        autofocus
      />
      <div class="app-list">
        {#if loading}
          <div class="state-msg">Loading installed apps...</div>
        {:else if filtered.length === 0}
          <div class="state-msg">No apps found</div>
        {:else}
          {#each filtered as app}
            <button class="app-item" onclick={() => selectApp(app)} title={app.exec}>
              <span class="app-name">{app.name}</span>
              <span class="app-exec">{app.exec}</span>
            </button>
          {/each}
        {/if}
      </div>
    {:else}
      <div class="browse-area">
        <button class="btn btn-secondary" onclick={handleBrowse}>
          Choose file...
        </button>
        {#if browsePath}
          <p class="browse-path">{browsePath}</p>
          <div class="field">
            <label for="browse-name">Display name</label>
            <input id="browse-name" type="text" bind:value={browseName} placeholder="My Editor" />
          </div>
          <button class="btn btn-primary" onclick={handleBrowseAdd} disabled={!browseName}>
            Add Editor
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 1100;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    width: 400px;
    max-width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px 10px;
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--color-text);
    border-bottom: 1px solid var(--color-border);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.8rem;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .tabs {
    display: flex;
    gap: 2px;
    background: var(--color-bg);
    margin: 10px 12px 0;
    border-radius: 7px;
    padding: 3px;
  }

  .tab {
    flex: 1;
    padding: 5px 10px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .tab.active {
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }

  .search-input {
    margin: 10px 12px 6px;
    padding: 7px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.875rem;
    outline: none;
  }

  .search-input:focus {
    border-color: var(--color-accent);
  }

  .app-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 6px 10px;
  }

  .app-item {
    display: flex;
    flex-direction: column;
    gap: 1px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .app-item:hover {
    background: var(--color-hover);
  }

  .app-name {
    font-size: 0.875rem;
    color: var(--color-text);
    font-weight: 500;
  }

  .app-exec {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .state-msg {
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 0.85rem;
    padding: 24px;
  }

  .browse-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
  }

  .browse-path {
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    word-break: break-all;
    background: var(--color-bg);
    padding: 6px 8px;
    border-radius: 5px;
    border: 1px solid var(--color-border);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field label {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .field input {
    padding: 7px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.875rem;
    outline: none;
  }

  .field input:focus {
    border-color: var(--color-accent);
  }

  .btn {
    padding: 7px 16px;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: filter 0.15s;
    align-self: flex-start;
  }

  .btn:hover:not(:disabled) { filter: brightness(0.92); }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-primary { background: var(--color-accent); color: white; }
  .btn-secondary {
    background: var(--color-hover);
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }
</style>
