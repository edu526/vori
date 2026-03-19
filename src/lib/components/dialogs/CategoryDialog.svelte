<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addCategory, addSubcategory, updateCategory, updateSubcategory } from '$lib/api/commands';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';

  const ICON_OPTIONS = ['folder', 'code', 'star', 'heart', 'briefcase'];

  const payload = $derived(
    dialogStore.current?.type === 'category' ? dialogStore.current : null,
  );

  const isOpen = $derived(!!payload);
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
        const existing = configStore.categories[payload.parentKey]?.subcategories[payload.key];
        description = existing?.description ?? '';
        icon = existing?.icon ?? 'folder';
      } else {
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
    if (!key) { keyError = 'Name is required.'; return false; }
    if (!keyPattern.test(key)) { keyError = 'Only letters, numbers, underscores and hyphens (no spaces).'; return false; }
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
          await updateSubcategory(payload.parentKey, payload.key, { description, icon });
        } else {
          const existing = configStore.categories[payload.key];
          await updateCategory(payload.key, { ...existing, description, icon });
        }
      }
      await configStore.load();
      navigationStore.refresh(
        configStore.categories, configStore.projects,
        configStore.files, configStore.favorites, configStore.recents,
      );
      dialogStore.close();
    } catch (e) {
      keyError = String(e);
    }
  }
</script>

<Dialog open={isOpen} onOpenChange={(o) => { if (!o) dialogStore.close(); }}>
  <DialogContent class="w-[420px] max-w-[90vw]" showCloseButton={false}>
    <DialogHeader>
      <DialogTitle>
        {#if isEdit && isSubcategory}Edit Subcategory
        {:else if isEdit}Edit Category
        {:else if isSubcategory}Add Subcategory
        {:else}Add Category{/if}
      </DialogTitle>
    </DialogHeader>

    <div class="fields">
      <div class="field">
        <Label for="cat-key">Name (key)</Label>
        <Input
          id="cat-key"
          bind:value={key}
          oninput={validateKey}
          disabled={isEdit}
          placeholder="my-category"
        />
        {#if keyError}<p class="error-msg">{keyError}</p>{/if}
      </div>

      <div class="field">
        <Label for="cat-description">Description</Label>
        <textarea
          id="cat-description"
          bind:value={description}
          rows="3"
          placeholder="Optional description"
          class="native-textarea"
        ></textarea>
      </div>

      <div class="field">
        <Label for="cat-icon">Icon</Label>
        <select id="cat-icon" bind:value={icon} class="native-select">
          {#each ICON_OPTIONS as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
    </div>

    <DialogFooter>
      <Button variant="ghost" onclick={() => dialogStore.close()}>Cancel</Button>
      <Button onclick={handleSave}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<style>
  .fields { display: flex; flex-direction: column; gap: 12px; }
  .field { display: flex; flex-direction: column; gap: 4px; }
  .error-msg { font-size: 0.8rem; color: #e53e3e; margin: 0; }

  .native-textarea,
  .native-select {
    width: 100%;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.875rem;
    font-family: inherit;
    padding: 7px 10px;
    outline: none;
    transition: border-color 0.15s;
  }
  .native-textarea:focus,
  .native-select:focus { border-color: var(--color-accent); }
  .native-textarea { resize: vertical; min-height: 68px; }
</style>
