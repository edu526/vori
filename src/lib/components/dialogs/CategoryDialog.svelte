<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addCategory, updateCategory, setClaudeProfile } from '$lib/api/commands';
  import { open } from '@tauri-apps/plugin-dialog';
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
  let sourcePath = $state<string>('');
  let claudeProfile = $state<string>('');
  let keyError = $state('');

  const profileNames = $derived(
    Object.keys(configStore.preferences.claude_profiles ?? {}).sort((a, b) => a.localeCompare(b)),
  );

  // All category keys, sorted — used for parent dropdown
  const categoryOptions = $derived(
    Object.keys(configStore.categories).sort((a, b) => a.localeCompare(b)),
  );

  $effect(() => {
    if (!payload) return;

    if (payload.mode === 'edit') {
      key = payload.key;
      selectedParent = configStore.categories[payload.key]?.parent ?? '';
      sourcePath = configStore.categories[payload.key]?.source_path ?? '';
      claudeProfile = configStore.categories[payload.key]?.claude_profile ?? '';
    } else {
      key = '';
      selectedParent = payload.parentKey ?? '';
      sourcePath = '';
      claudeProfile = '';
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

  async function handleBrowse() {
    const picked = await open({ directory: true });
    if (typeof picked === 'string') sourcePath = picked;
  }

  async function handleSave() {
    if (!validateKey() || !payload) return;
    const parent = selectedParent || null;
    const trimmedSource = sourcePath.trim() || null;
    try {
      if (payload.mode === 'add') {
        await addCategory(key, parent, trimmedSource);
      } else if (payload.mode === 'edit') {
        await updateCategory(payload.key, parent, trimmedSource);
      }
      // Profile is set separately: add/update_category never touch it.
      const previous = payload.mode === 'edit' ? (configStore.categories[key]?.claude_profile ?? '') : '';
      if (claudeProfile !== previous) {
        await setClaudeProfile('category', key, claudeProfile || null);
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

      <div class="field">
        <Label for="cat-source">Folder (optional)</Label>
        <div class="path-row">
          <Input
            id="cat-source"
            class="path-input"
            bind:value={sourcePath}
            placeholder="/path/to/folder"
          />
          <Button type="button" variant="outline" size="sm" onclick={handleBrowse}>Browse</Button>
        </div>
        <span class="hint">Bind a folder to auto-detect workspaces and enable Refresh Import Tree</span>
      </div>

      <div class="field">
        <Label for="cat-claude">Claude profile</Label>
        {#if profileNames.length > 0}
          <select id="cat-claude" bind:value={claudeProfile} class="native-select">
            <option value="">Inherit from parent</option>
            {#each profileNames as name}
              <option value={name}>{name}</option>
            {/each}
          </select>
          <span class="hint">Applies to everything inside when opened in an editor or terminal</span>
        {:else}
          <span class="hint">No profiles yet — create one in Preferences → Claude</span>
        {/if}
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
  .error-msg { font-size: var(--text-sm); color: #e53e3e; margin: 0; }

  .path-row { display: flex; gap: 6px; align-items: stretch; }
  .hint { font-size: var(--text-xs); color: var(--color-text-secondary); font-style: italic; }

  .native-select {
    width: 100%;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: var(--text-base);
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
