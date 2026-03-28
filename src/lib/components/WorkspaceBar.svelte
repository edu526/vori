<script lang="ts">
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { openWorkspaceInEditor } from '$lib/api/commands';

  const EDITOR_LABELS: Record<string, string> = {
    'vscode': 'VSCode', 'vscode-insiders': 'VSCode Insiders', 'cursor': 'Cursor',
    'windsurf': 'Windsurf', 'kiro': 'Kiro', 'zed': 'Zed', 'fleet': 'Fleet',
    'sublime': 'Sublime Text', 'neovim': 'Neovim', 'vim': 'Vim', 'emacs': 'Emacs',
  };

  function editorLabel(key: string): string {
    return EDITOR_LABELS[key] ?? key;
  }

  let selectedEditor = $state('');

  $effect(() => {
    selectedEditor = configStore.preferences.default_editor;
  });

  let entries = $derived([...navigationStore.workspaceSelection.entries()]);

  async function openWorkspace() {
    const paths = entries.map(([, v]) => v.path);
    const editor = selectedEditor || configStore.preferences.default_editor;
    await openWorkspaceInEditor(paths, editor);
    navigationStore.clearWorkspaceSelection();
  }

  let availableEditors = $derived(Object.keys(configStore.preferences.editors_available ?? {}));
</script>

{#if navigationStore.workspaceSelection.size > 0}
  <div class="workspace-bar" role="toolbar" aria-label="Workspace selection">
    <div class="chips">
      {#each entries as [key, entry] (key)}
        <span class="chip">
          {entry.label}
          <button
            class="chip-remove"
            onclick={() => navigationStore.toggleWorkspaceItem(key, entry.path, entry.label)}
            aria-label="Remove {entry.label}"
          >×</button>
        </span>
      {/each}
    </div>

    <div class="actions">
      {#if availableEditors.length > 1}
        <select class="editor-select" bind:value={selectedEditor}>
          {#each availableEditors as key (key)}
            <option value={key}>{editorLabel(key)}</option>
          {/each}
        </select>
      {/if}

      <button class="btn-open" onclick={openWorkspace}>
        Open workspace
        <span class="count">{navigationStore.workspaceSelection.size}</span>
      </button>

      <button class="btn-clear" onclick={() => navigationStore.clearWorkspaceSelection()} title="Clear selection">
        ×
      </button>
    </div>
  </div>
{/if}

<style>
  .workspace-bar {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
    z-index: 200;
    max-width: calc(100vw - 40px);
    animation: slide-up 0.15s ease-out;
  }

  @keyframes slide-up {
    from { opacity: 0; transform: translateX(-50%) translateY(8px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0); }
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    max-width: 400px;
  }

  .chip {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    color: var(--color-accent);
    border-radius: 99px;
    font-size: 0.78rem;
    font-weight: 500;
    white-space: nowrap;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .chip-remove {
    flex-shrink: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    font-size: 0.9rem;
    opacity: 0.7;
    line-height: 1;
  }

  .chip-remove:hover {
    opacity: 1;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .editor-select {
    padding: 4px 6px;
    font-size: 0.8rem;
    border: 1px solid var(--color-border);
    border-radius: 5px;
    background: var(--color-surface);
    color: var(--color-text);
    cursor: pointer;
  }

  .btn-open {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
  }

  .btn-open:hover {
    opacity: 0.88;
  }

  .count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    background: rgba(255, 255, 255, 0.25);
    border-radius: 99px;
    font-size: 0.72rem;
    font-weight: 600;
  }

  .btn-clear {
    padding: 4px 8px;
    background: none;
    border: 1px solid var(--color-border);
    border-radius: 5px;
    color: var(--color-text-secondary);
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
  }

  .btn-clear:hover {
    background: var(--color-hover);
    color: var(--color-text);
  }
</style>
