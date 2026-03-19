<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addFile, updateFile } from '$lib/api/commands';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';

  const payload = $derived(
    dialogStore.current?.type === 'file' ? dialogStore.current : null,
  );

  const isOpen = $derived(!!payload);
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
        if (!key) key = result.split('/').pop()?.replace(/\.[^.]+$/, '') ?? '';
      }
    } catch (e) {
      console.error('Failed to open file picker:', e);
    }
  }

  async function handleSave() {
    if (!key) { keyError = 'Name is required.'; return; }
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
      <DialogTitle>{isEdit ? 'Edit File' : 'Add File'}</DialogTitle>
    </DialogHeader>

    <div class="fields">
      <div class="field">
        <Label for="file-key">Name</Label>
        <Input id="file-key" bind:value={key} disabled={isEdit} placeholder="my-config" />
        {#if keyError}<p class="error-msg">{keyError}</p>{/if}
      </div>

      <div class="field">
        <Label for="file-path">Path</Label>
        <div class="path-row">
          <Input id="file-path" bind:value={path} placeholder="/home/user/.config/file.toml" />
          <Button variant="outline" size="sm" onclick={handleBrowse}>Browse</Button>
        </div>
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
</style>
