<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { addProject, gitClone } from '$lib/api/commands';
  import { open } from '@tauri-apps/plugin-dialog';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';

  const LAST_DEST_KEY = 'vori.cloneDestination';

  const payload = $derived(dialogStore.current?.type === 'clone' ? dialogStore.current : null);
  const isOpen = $derived(!!payload);

  let url = $state('');
  let destination = $state('');
  let selectedParent = $state('');
  let error = $state('');
  let cloning = $state(false);

  const categoryOptions = $derived(
    Object.keys(configStore.categories).sort((a, b) => a.localeCompare(b)),
  );

  function rememberedDestination(): string {
    try {
      return localStorage.getItem(LAST_DEST_KEY) ?? '';
    } catch {
      return '';
    }
  }

  $effect(() => {
    if (!payload) return;
    url = '';
    error = '';
    cloning = false;
    selectedParent = payload.parentKey ?? '';
    // A category bound to a folder is the natural place to clone into.
    const bound = payload.parentKey ? configStore.categories[payload.parentKey]?.source_path : null;
    destination = bound || rememberedDestination();
  });

  async function handleBrowse() {
    try {
      const result = await open({ directory: true });
      if (typeof result === 'string') destination = result;
    } catch (e) {
      console.error('Failed to open directory picker:', e);
    }
  }

  function uniqueKey(base: string): string {
    if (!configStore.projects[base]) return base;
    let n = 2;
    while (configStore.projects[`${base}-${n}`]) n++;
    return `${base}-${n}`;
  }

  async function handleClone() {
    if (cloning) return;
    if (!url.trim()) { error = t('clone.urlRequired'); return; }
    if (!destination.trim()) { error = t('clone.destRequired'); return; }
    if (!selectedParent) { error = t('dialog.parentRequired'); return; }
    error = '';
    cloning = true;
    try {
      const path = await gitClone(url.trim(), destination.trim());
      try {
        localStorage.setItem(LAST_DEST_KEY, destination.trim());
      } catch {
        /* remembering the folder is a convenience only */
      }
      const name = path.split(/[\\/]/).filter(Boolean).pop() ?? 'project';
      await addProject(uniqueKey(name), { path, parent: selectedParent });
      await configStore.load();
      navigationStore.refresh(
        configStore.categories, configStore.projects,
        configStore.files, configStore.favorites, configStore.recents,
      );
      dialogStore.close();
    } catch (e) {
      error = String(e);
      cloning = false;
    }
  }
</script>

<Dialog open={isOpen} onOpenChange={(o) => { if (!o && !cloning) dialogStore.close(); }}>
  <DialogContent class="w-[460px] max-w-[90vw]" showCloseButton={false}>
    <DialogHeader>
      <DialogTitle>{t('clone.title')}</DialogTitle>
    </DialogHeader>

    <div class="fields">
      <div class="field">
        <Label for="clone-url">{t('clone.url')}</Label>
        <Input
          id="clone-url"
          bind:value={url}
          disabled={cloning}
          placeholder="https://github.com/user/repo.git"
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') handleClone(); }}
        />
      </div>

      <div class="field">
        <Label for="clone-dest">{t('clone.into')}</Label>
        <div class="path-row">
          <Input id="clone-dest" bind:value={destination} disabled={cloning} placeholder="/home/user/projects" />
          <Button variant="outline" size="sm" onclick={handleBrowse} disabled={cloning}>{t('common.browse')}</Button>
        </div>
        <span class="hint">{t('clone.intoHint')}</span>
      </div>

      <div class="field">
        <Label for="clone-parent">{t('field.category')}</Label>
        <select id="clone-parent" bind:value={selectedParent} disabled={cloning} class="native-select">
          <option value="">{t('dialog.selectCategory')}</option>
          {#each categoryOptions as catKey}
            <option value={catKey}>{catKey}</option>
          {/each}
        </select>
      </div>

      {#if error}<p class="error-msg">{error}</p>{/if}
    </div>

    <DialogFooter>
      <Button variant="ghost" onclick={() => dialogStore.close()} disabled={cloning}>{t('common.cancel')}</Button>
      <Button onclick={handleClone} disabled={cloning}>{cloning ? t('clone.cloning') : t('clone.action')}</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<style>
  .fields { display: flex; flex-direction: column; gap: 12px; }
  .field { display: flex; flex-direction: column; gap: 4px; }
  .error-msg { font-size: var(--text-sm); color: #e53e3e; margin: 0; word-break: break-word; }

  .path-row { display: flex; gap: 6px; align-items: center; }
  .path-row :global(input) { flex: 1; }
  .hint { font-size: var(--text-xs); color: var(--color-text-secondary); }

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
  .native-select:disabled { opacity: 0.6; }
</style>
