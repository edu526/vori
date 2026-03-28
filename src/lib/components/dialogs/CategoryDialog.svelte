<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addCategory, updateCategory } from '$lib/api/commands';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';

  const payload = $derived(
    dialogStore.current?.type === 'category' ? dialogStore.current : null,
  );

  const isOpen = $derived(!!payload);
  const isEdit = $derived(payload?.mode === 'edit');

  let key = $state('');
  let selectedParent = $state<string>('');
  let keyError = $state('');

  // All category keys, sorted — used for parent dropdown
  const categoryOptions = $derived(
    Object.keys(configStore.categories).sort((a, b) => a.localeCompare(b)),
  );

  $effect(() => {
    if (!payload) return;

    if (payload.mode === 'edit') {
      key = payload.key;
      selectedParent = configStore.categories[payload.key]?.parent ?? '';
    } else {
      key = '';
      selectedParent = payload.parentKey ?? '';
    }
    keyError = '';
  });

  const keyPattern = /^[a-zA-Z0-9_/-]+$/;

  function validateKey() {
    if (!key) { keyError = 'Name is required.'; return false; }
    if (!keyPattern.test(key)) { keyError = 'Only letters, numbers, underscores, hyphens and slashes.'; return false; }
    keyError = '';
    return true;
  }

  async function handleSave() {
    if (!validateKey() || !payload) return;
    const parent = selectedParent || null;
    try {
      if (payload.mode === 'add') {
        await addCategory(key, parent);
      } else if (payload.mode === 'edit') {
        await updateCategory(payload.key, parent);
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
      <DialogTitle>{isEdit ? 'Edit Category' : 'Add Category'}</DialogTitle>
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
        <Label for="cat-parent">Parent category</Label>
        <select id="cat-parent" bind:value={selectedParent} class="native-select">
          <option value="">(none — root)</option>
          {#each categoryOptions as catKey}
            {#if !(isEdit && payload?.mode === 'edit' && catKey === payload.key)}
              <option value={catKey}>{catKey}</option>
            {/if}
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
    appearance: none;
    -webkit-appearance: none;
    color-scheme: light dark;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%236b7080' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    padding-right: 28px;
  }
  .native-select:focus { border-color: var(--color-accent); }
</style>
