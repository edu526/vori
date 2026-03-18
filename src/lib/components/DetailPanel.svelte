<script lang="ts">
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { openProjectInEditor, openFileInEditor, openInTerminal } from '$lib/api/commands';

  let errorMessage = $state<string | null>(null);
  let errorTimer: ReturnType<typeof setTimeout> | null = null;

  function showError(msg: string) {
    if (errorTimer) clearTimeout(errorTimer);
    errorMessage = msg;
    errorTimer = setTimeout(() => {
      errorMessage = null;
      errorTimer = null;
    }, 3000);
  }

  $effect(() => {
    return () => {
      if (errorTimer) clearTimeout(errorTimer);
    };
  });

  let selectedItem = $derived((() => {
    const cols = navigationStore.columns;
    for (let i = cols.length - 1; i >= 0; i--) {
      if (cols[i].selectedKey) {
        const item = cols[i].items.find(it => it.key === cols[i].selectedKey);
        if (item && item.type !== 'section-header') return item;
      }
    }
    return null;
  })());

  let projectCount = $derived((() => {
    if (!selectedItem) return 0;
    const key = selectedItem.key;
    const type = selectedItem.type;
    if (type === 'category') {
      return Object.values(configStore.projects).filter(p => p.category === key).length;
    }
    if (type === 'subcategory') {
      return Object.values(configStore.projects).filter(
        p => p.subcategory === key && p.category === selectedItem?.categoryKey
      ).length;
    }
    return 0;
  })());

  let defaultEditor = $derived(configStore.preferences.default_editor);

  function editorLabel(editorId: string): string {
    if (editorId === 'vscode') return 'VSCode';
    if (editorId === 'kiro') return 'Kiro';
    return editorId;
  }

  async function handleOpenInEditor(path: string, editor: string) {
    try {
      await openProjectInEditor(path, editor);
    } catch (e) {
      showError(String(e));
    }
  }

  async function handleOpenFile(path: string) {
    try {
      await openFileInEditor(path, configStore.preferences.default_text_editor);
    } catch (e) {
      showError(String(e));
    }
  }

  async function handleOpenTerminal(path: string) {
    try {
      await openInTerminal(path);
    } catch (e) {
      showError(String(e));
    }
  }
</script>

<aside class="detail-panel">
  {#if selectedItem === null}
    <div class="empty-state">
      <svg
        class="empty-icon"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
      <p class="empty-text">Select a project to open it</p>
    </div>

  {:else if selectedItem.type === 'project'}
    <div class="item-header">
      <h2 class="item-name">{selectedItem.label}</h2>
      {#if selectedItem.path}
        <p class="item-path" title={selectedItem.path}>{selectedItem.path}</p>
      {/if}
    </div>

    <div class="actions">
      {#if selectedItem.path}
        <button
          class="btn btn-primary"
          onclick={() => handleOpenInEditor(selectedItem!.path!, defaultEditor)}
        >
          Open in {editorLabel(defaultEditor)}
        </button>

        {#if defaultEditor !== 'vscode'}
          <button
            class="btn btn-secondary"
            onclick={() => handleOpenInEditor(selectedItem!.path!, 'vscode')}
          >
            Open in VSCode
          </button>
        {/if}

        {#if defaultEditor !== 'kiro'}
          <button
            class="btn btn-secondary"
            onclick={() => handleOpenInEditor(selectedItem!.path!, 'kiro')}
          >
            Open in Kiro
          </button>
        {/if}

        <button
          class="btn btn-secondary"
          onclick={() => handleOpenTerminal(selectedItem!.path!)}
        >
          Open Terminal here
        </button>
      {/if}
    </div>

    {#if errorMessage}
      <p class="error-message">{errorMessage}</p>
    {/if}

  {:else if selectedItem.type === 'file'}
    <div class="item-header">
      <h2 class="item-name">{selectedItem.label}</h2>
      {#if selectedItem.path}
        <p class="item-path" title={selectedItem.path}>{selectedItem.path}</p>
      {/if}
    </div>

    <div class="actions">
      {#if selectedItem.path}
        <button
          class="btn btn-primary"
          onclick={() => handleOpenFile(selectedItem!.path!)}
        >
          Open File
        </button>
      {/if}
    </div>

    {#if errorMessage}
      <p class="error-message">{errorMessage}</p>
    {/if}

  {:else if selectedItem.type === 'category' || selectedItem.type === 'subcategory'}
    <div class="item-header">
      <h2 class="item-name">{selectedItem.label}</h2>
      <p class="item-meta">{projectCount} {projectCount === 1 ? 'project' : 'projects'}</p>
    </div>
  {/if}
</aside>

<style>
  .detail-panel {
    width: 260px;
    flex-shrink: 0;
    border-left: 1px solid var(--color-border);
    background: var(--color-surface);
    padding: 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--color-text-secondary, #888);
  }

  .empty-icon {
    width: 56px;
    height: 56px;
    opacity: 0.4;
  }

  .empty-text {
    font-size: 13px;
    text-align: center;
    margin: 0;
    opacity: 0.7;
  }

  .item-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .item-name {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
    word-break: break-word;
    color: var(--color-text, inherit);
  }

  .item-path {
    font-size: 11px;
    color: var(--color-text-secondary, #888);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-meta {
    font-size: 12px;
    color: var(--color-text-secondary, #888);
    margin: 0;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .btn {
    width: 100%;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    text-align: center;
    transition: opacity 0.15s;
  }

  .btn:hover {
    opacity: 0.85;
  }

  .btn:active {
    opacity: 0.7;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
    padding: 8px 12px;
    font-weight: 600;
  }

  .btn-secondary {
    background: var(--color-hover);
    color: var(--color-text, inherit);
    padding: 7px 12px;
    font-weight: 400;
  }

  .error-message {
    font-size: 11px;
    color: #e05252;
    margin: 0;
    word-break: break-word;
  }
</style>
