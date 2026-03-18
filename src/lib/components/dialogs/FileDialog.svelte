<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addFile, updateFile } from '$lib/api/commands';

  const payload = $derived(
    dialogStore.current?.type === 'file' ? dialogStore.current : null,
  );

  const isEdit = $derived(payload?.mode === 'edit');

  let key = $state('');
  let path = $state('');
  let keyError = $state('');

  $effect(() => {
    if (!payload) return;

    if (payload.mode === 'edit') {
      const existing = configStore.files[payload.key];
      key = payload.key;
      path = existing?.path ?? '';
    } else {
      key = '';
      path = '';
    }
    keyError = '';
  });

  async function handleBrowse() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const result = await open({ directory: false });
      if (typeof result === 'string') {
        path = result;
        // Auto-fill name from filename if empty
        if (!key) {
          key = result.split('/').pop()?.replace(/\.[^.]+$/, '') ?? '';
        }
      }
    } catch (e) {
      console.error('Failed to open file picker:', e);
    }
  }

  async function handleSave() {
    if (!key) {
      keyError = 'Name is required.';
      return;
    }
    if (!payload) return;

    keyError = '';

    try {
      if (payload.mode === 'edit') {
        await updateFile(payload.key, { path });
      } else {
        await addFile(key, { path });
      }
      await configStore.load();
      navigationStore.refresh(
        configStore.categories,
        configStore.projects,
        configStore.files,
        configStore.favorites,
        configStore.recents,
      );
      dialogStore.close();
    } catch (e) {
      keyError = String(e);
    }
  }

  function handleBackdropKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') dialogStore.close();
  }

  function stopPropagation(e: MouseEvent | KeyboardEvent) {
    e.stopPropagation();
  }
</script>

{#if payload}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => dialogStore.close()} onkeydown={handleBackdropKeydown}>
    <div class="dialog" role="dialog" aria-modal="true" tabindex="-1" onclick={stopPropagation} onkeydown={stopPropagation}>
      <div class="dialog-header">
        {isEdit ? 'Edit File' : 'Add File'}
      </div>

      <div class="field">
        <label for="file-key">Name</label>
        <input
          id="file-key"
          type="text"
          bind:value={key}
          disabled={isEdit}
          placeholder="my-config"
        />
        {#if keyError}
          <span class="error-msg">{keyError}</span>
        {/if}
      </div>

      <div class="field">
        <label for="file-path">Path</label>
        <div class="path-row">
          <input
            id="file-path"
            type="text"
            bind:value={path}
            placeholder="/home/user/.config/file.toml"
          />
          <button class="btn btn-secondary browse-btn" onclick={handleBrowse}>Browse...</button>
        </div>
      </div>

      <div class="dialog-actions">
        <button class="btn btn-secondary" onclick={() => dialogStore.close()}>Cancel</button>
        <button class="btn btn-primary" onclick={handleSave}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    padding: 24px;
    width: 420px;
    max-width: 90vw;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .dialog-header {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text);
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
  }

  .field input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .field input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .path-row {
    display: flex;
    gap: 6px;
  }

  .path-row input {
    flex: 1;
    padding: 7px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.875rem;
  }

  .path-row input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .browse-btn {
    white-space: nowrap;
    flex-shrink: 0;
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 8px;
  }

  .btn {
    padding: 7px 16px;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
  }

  .btn-secondary {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .error-msg {
    color: #e53e3e;
    font-size: 0.8rem;
  }
</style>
