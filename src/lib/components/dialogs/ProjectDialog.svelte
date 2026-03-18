<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { addProject, updateProject } from '$lib/api/commands';

  const payload = $derived(
    dialogStore.current?.type === 'project' ? dialogStore.current : null,
  );

  const isEdit = $derived(payload?.mode === 'edit');

  let key = $state('');
  let path = $state('');
  let selectedCategory = $state('');
  let selectedSubcategory = $state('');
  let keyError = $state('');

  const subcategories = $derived(
    selectedCategory && configStore.categories[selectedCategory]
      ? Object.keys(configStore.categories[selectedCategory].subcategories)
      : [],
  );

  $effect(() => {
    if (!payload) return;

    if (payload.mode === 'edit') {
      const existing = configStore.projects[payload.key];
      key = payload.key;
      path = existing?.path ?? '';
      selectedCategory = existing?.category ?? '';
      selectedSubcategory = existing?.subcategory ?? '';
    } else {
      key = '';
      path = '';
      selectedCategory = payload.categoryKey ?? '';
      selectedSubcategory = payload.subcategoryKey ?? '';
    }
    keyError = '';
  });

  $effect(() => {
    // Reset subcategory when category changes
    if (selectedCategory && !subcategories.includes(selectedSubcategory)) {
      selectedSubcategory = '';
    }
  });

  async function handleBrowse() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const result = await open({ directory: true });
      if (typeof result === 'string') {
        path = result;
      }
    } catch (e) {
      console.error('Failed to open directory picker:', e);
    }
  }

  async function handleSave() {
    if (!key) {
      keyError = 'Name is required.';
      return;
    }
    if (!payload) return;

    keyError = '';

    const projectData = {
      path,
      category: selectedCategory,
      subcategory: selectedSubcategory || undefined,
    };

    try {
      if (payload.mode === 'edit') {
        await updateProject(payload.key, projectData);
      } else {
        await addProject(key, projectData);
      }
      await configStore.load();
      dialogStore.close();
    } catch (e) {
      keyError = String(e);
    }
  }

  function handleBackdropKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') dialogStore.close();
  }

  function stopPropagation(e: MouseEvent) {
    e.stopPropagation();
  }
</script>

{#if payload}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => dialogStore.close()} onkeydown={handleBackdropKeydown}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dialog" onclick={stopPropagation}>
      <div class="dialog-header">
        {isEdit ? 'Edit Project' : 'Add Project'}
      </div>

      <div class="field">
        <label for="proj-key">Name</label>
        <input
          id="proj-key"
          type="text"
          bind:value={key}
          disabled={isEdit}
          placeholder="my-project"
        />
        {#if keyError}
          <span class="error-msg">{keyError}</span>
        {/if}
      </div>

      <div class="field">
        <label for="proj-path">Path</label>
        <div class="path-row">
          <input
            id="proj-path"
            type="text"
            bind:value={path}
            placeholder="/home/user/projects/my-project"
          />
          <button class="btn btn-secondary browse-btn" onclick={handleBrowse}>Browse...</button>
        </div>
      </div>

      <div class="field">
        <label for="proj-category">Category</label>
        <select id="proj-category" bind:value={selectedCategory}>
          <option value="">(none)</option>
          {#each Object.keys(configStore.categories) as catKey}
            <option value={catKey}>{catKey}</option>
          {/each}
        </select>
      </div>

      {#if subcategories.length > 0}
        <div class="field">
          <label for="proj-subcategory">Subcategory</label>
          <select id="proj-subcategory" bind:value={selectedSubcategory}>
            <option value="">(none)</option>
            {#each subcategories as subKey}
              <option value={subKey}>{subKey}</option>
            {/each}
          </select>
        </div>
      {/if}

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

  .field input,
  .field textarea,
  .field select {
    padding: 7px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.875rem;
  }

  .field input:focus,
  .field textarea:focus,
  .field select:focus {
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
