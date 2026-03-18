<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addCategory, addSubcategory, updateCategory, updateSubcategory } from '$lib/api/commands';

  const ICON_OPTIONS = ['folder', 'code', 'star', 'heart', 'briefcase'];

  const payload = $derived(
    dialogStore.current?.type === 'category' ? dialogStore.current : null,
  );

  const isEdit = $derived(payload?.mode === 'edit');
  const isSubcategory = $derived(
    (payload?.mode === 'add' && !!payload.parentKey) ||
    (payload?.mode === 'edit' && !!payload.parentKey),
  );

  let key = $state('');
  let description = $state('');
  let icon = $state('folder');
  let keyError = $state('');

  $effect(() => {
    if (!payload) return;

    if (payload.mode === 'edit') {
      key = payload.key;
      if (payload.parentKey) {
        // Subcategory edit
        const existing = configStore.categories[payload.parentKey]?.subcategories[payload.key];
        description = existing?.description ?? '';
        icon = existing?.icon ?? 'folder';
      } else {
        // Category edit
        const existing = configStore.categories[payload.key];
        description = existing?.description ?? '';
        icon = existing?.icon ?? 'folder';
      }
    } else {
      key = '';
      description = '';
      icon = 'folder';
    }
    keyError = '';
  });

  const keyPattern = /^[a-zA-Z0-9_-]+$/;

  function validateKey() {
    if (!key) {
      keyError = 'Name is required.';
      return false;
    }
    if (!keyPattern.test(key)) {
      keyError = 'Only letters, numbers, underscores and hyphens are allowed (no spaces).';
      return false;
    }
    keyError = '';
    return true;
  }

  async function handleSave() {
    if (!validateKey() || !payload) return;

    try {
      if (payload.mode === 'add') {
        if (payload.parentKey) {
          await addSubcategory(payload.parentKey, key, { description, icon });
        } else {
          await addCategory(key, { description, icon, subcategories: {} });
        }
      } else if (payload.mode === 'edit') {
        if (payload.parentKey) {
          // Subcategory edit
          await updateSubcategory(payload.parentKey, payload.key, { description, icon });
        } else {
          // Category edit
          const existing = configStore.categories[payload.key];
          await updateCategory(payload.key, {
            ...existing,
            description,
            icon,
          });
        }
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
        {#if isEdit && isSubcategory}
          Edit Subcategory
        {:else if isEdit}
          Edit Category
        {:else if isSubcategory}
          Add Subcategory
        {:else}
          Add Category
        {/if}
      </div>

      <div class="field">
        <label for="cat-key">Name (key)</label>
        <input
          id="cat-key"
          type="text"
          bind:value={key}
          oninput={validateKey}
          disabled={isEdit}
          placeholder="my-category"
        />
        {#if keyError}
          <span class="error-msg">{keyError}</span>
        {/if}
      </div>

      <div class="field">
        <label for="cat-description">Description</label>
        <textarea
          id="cat-description"
          bind:value={description}
          rows="3"
          placeholder="Optional description"
        ></textarea>
      </div>

      <div class="field">
        <label for="cat-icon">Icon</label>
        <select id="cat-icon" bind:value={icon}>
          {#each ICON_OPTIONS as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
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
