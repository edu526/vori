<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addProject, updateProject } from '$lib/api/commands';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';

  const payload = $derived(
    dialogStore.current?.type === 'project' ? dialogStore.current : null,
  );

  const isOpen = $derived(!!payload);
  const isEdit = $derived(payload?.mode === 'edit');

  let key = $state('');
  let path = $state('');
  let selectedParent = $state('');
  let keyError = $state('');

  const categoryOptions = $derived(
    Object.keys(configStore.categories).sort((a, b) => a.localeCompare(b)),
  );

  $effect(() => {
    if (!payload) return;
    if (payload.mode === 'edit') {
      const existing = configStore.projects[payload.key];
      key = payload.key;
      path = existing?.path ?? '';
      selectedParent = existing?.parent ?? '';
    } else {
      key = '';
      path = '';
      selectedParent = payload.parentKey ?? '';
    }
    keyError = '';
  });

  async function handleBrowse() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const result = await open({ directory: true });
      if (typeof result === 'string') path = result;
    } catch (e) {
      console.error('Failed to open directory picker:', e);
    }
  }

  async function handleSave() {
    if (!key) { keyError = 'Name is required.'; return; }
    if (!selectedParent) { keyError = 'Parent category is required.'; return; }
    if (!payload) return;
    keyError = '';
    try {
      const projectData = { path, parent: selectedParent };
      if (payload.mode === 'edit') {
        await updateProject(payload.key, projectData);
      } else {
        await addProject(key, projectData);
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
      <DialogTitle>{isEdit ? 'Edit Project' : 'Add Project'}</DialogTitle>
    </DialogHeader>

    <div class="fields">
      <div class="field">
        <Label for="proj-key">Name</Label>
        <Input id="proj-key" bind:value={key} disabled={isEdit} placeholder="my-project" />
        {#if keyError}<p class="error-msg">{keyError}</p>{/if}
      </div>

      <div class="field">
        <Label for="proj-path">Path</Label>
        <div class="path-row">
          <Input id="proj-path" bind:value={path} placeholder="/home/user/projects/my-project" />
          <Button variant="outline" size="sm" onclick={handleBrowse}>Browse</Button>
        </div>
      </div>

      <div class="field">
        <Label for="proj-parent">Category</Label>
        <select id="proj-parent" bind:value={selectedParent} class="native-select">
          <option value="">(select category)</option>
          {#each categoryOptions as catKey}
            <option value={catKey}>{catKey}</option>
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

  .path-row { display: flex; gap: 6px; align-items: center; }
  .path-row :global(input) { flex: 1; }

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
