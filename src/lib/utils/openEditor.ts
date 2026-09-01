import { ask } from '@tauri-apps/plugin-dialog';
import { dialogStore } from '$lib/stores/dialogs.svelte';
import { editorStore } from '$lib/stores/editor.svelte';

export async function openEditor(filePath: string, fileName: string) {
  // Already open with the same file — nothing to do. Avoids a no-op
  // re-open that flashes the editor through the loading state.
  if (
    dialogStore.current?.type === 'editor' &&
    dialogStore.current.filePath === filePath
  ) {
    return;
  }

  if (
    editorStore.dirty &&
    editorStore.currentFilePath &&
    editorStore.currentFilePath !== filePath
  ) {
    const currentName = editorStore.currentFilePath.split(/[\\/]/).pop() ?? editorStore.currentFilePath;
    const ok = await ask(
      `Tienes cambios sin guardar en "${currentName}". ¿Cambiar de archivo de todos modos?`,
      { kind: 'warning' },
    );
    if (!ok) return;
  }
  dialogStore.open({ type: 'editor', filePath, fileName });
}