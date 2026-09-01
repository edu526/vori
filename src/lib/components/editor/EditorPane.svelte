<script lang="ts">
  import { onDestroy, untrack } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState, Prec } from '@codemirror/state';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { keymap } from '@codemirror/view';
  import { indentUnit } from '@codemirror/language';

  import { readTextFile, writeTextFile, writeTextFileElevated } from '$lib/api/commands';
  import { languageForPath } from '$lib/utils/textExtensions';
  import { configStore } from '$lib/stores/config.svelte';
  import { editorStore } from '$lib/stores/editor.svelte';
  import { ask } from '@tauri-apps/plugin-dialog';

  import { Button } from '$lib/components/ui/button';

  let { filePath, fileName, breadcrumb = [], onclose, onback, onnavigate }: {
    filePath: string;
    fileName: string;
    breadcrumb?: { label: string; depth: number }[];
    onclose: () => void;
    onback?: () => void;
    onnavigate?: (depth: number) => void;
  } = $props();

  let container: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let content = $state('');
  let loading = $state(true);
  let saving = $state(false);
  let dirty = $state(false);
  let saved = $state(false);
  let error = $state('');
  let mountedFor = $state<string | null>(null);
  let savedDoc = ''; // last saved/loaded content; used to clear dirty on undo-back-to-saved

  async function load() {
    loading = true;
    error = '';
    try {
      const text = await readTextFile(filePath);
      untrack(() => {
        content = text;
        savedDoc = text;
        dirty = false;
      });
    } catch (e) {
      error = String(e);
      content = '';
      savedDoc = '';
    } finally {
      loading = false;
    }
  }

  async function save() {
    saving = true;
    saved = false;
    error = '';
    try {
      await writeTextFile(filePath, content);
      savedDoc = content;
      dirty = false;
      saved = true;
      setTimeout(() => { saved = false; }, 1500);
    } catch (e) {
      const msg = String(e);
      if (/permission|denied|access|io error/i.test(msg)) {
        const ok = await ask(
          'Permisos insuficientes para guardar. ¿Reintentar como administrador?',
          { kind: 'warning' },
        );
        if (ok) {
          try {
            await writeTextFileElevated(filePath, content);
            savedDoc = content;
            dirty = false;
            saved = true;
            setTimeout(() => { saved = false; }, 1500);
          } catch (e2) {
            error = String(e2);
          }
        } else {
          error = msg;
        }
      } else {
        error = msg;
      }
    } finally {
      saving = false;
    }
  }

  async function cancel() {
    if (dirty) {
      const ok = await ask('Tienes cambios sin guardar. ¿Cerrar de todos modos?', {
        kind: 'warning',
      });
      if (!ok) return;
    }
    onclose();
  }

  $effect(() => {
    if (filePath) {
      dirty = false;
      saved = false;
      error = '';
      load();
    }
  });

  $effect(() => {
    if (!container || loading) return;
    if (mountedFor === filePath) return;

    const initial = untrack(() => content);
    const prefs = untrack(() => configStore.preferences);
    const tabSize = prefs?.editor_tab_size ?? 2;
    const fontSize = prefs?.editor_font_size ?? 13;
    const textWrap = prefs?.editor_text_wrap ?? false;

    view?.destroy();
    view = new EditorView({
      parent: container,
      state: EditorState.create({
        doc: initial,
        extensions: [
          basicSetup,
          oneDark,
          Prec.highest(
            keymap.of([
              {
                key: 'Mod-s',
                preventDefault: true,
                run: () => {
                  if (!saving && !loading && dirty) save();
                  return true;
                },
              },
              {
                key: 'Escape',
                preventDefault: true,
                run: () => { cancel(); return true; },
              },
            ]),
          ),
          languageForPath(filePath),
          indentUnit.of(' '.repeat(tabSize)),
          ...(textWrap ? [EditorView.lineWrapping] : []),
          EditorView.updateListener.of((u) => {
            if (u.docChanged) {
              content = u.state.doc.toString();
              // Dirty iff current doc differs from the last saved/loaded version.
              // This makes undo/redo back to the saved state correctly clear dirty.
              dirty = content !== savedDoc;
              if (dirty) saved = false;
            }
          }),
          EditorView.theme({
            '&': { height: '100%', fontSize: `${fontSize}px` },
            '.cm-scroller': {
              fontFamily: 'var(--font-mono, ui-monospace, Menlo, monospace)',
            },
          }),
        ],
      }),
    });
    view.focus();
    const focusView = () => {
      if (!view) return;
      view.focus();
      // Belt-and-suspenders: also focus the contentDOM directly in case
      // CodeMirror's focus() races with the split-view transition.
      try {
        view.contentDOM?.focus({ preventScroll: true });
      } catch {}
    };
    requestAnimationFrame(focusView);
    setTimeout(focusView, 50);
    setTimeout(focusView, 200);
    mountedFor = filePath;
  });

  $effect(() => {
    editorStore.setDirty(dirty);
    editorStore.setCurrentFilePath(filePath);
  });

  onDestroy(() => {
    view?.destroy();
    editorStore.setDirty(false);
    editorStore.setCurrentFilePath(null);
  });
</script>

<div class="editor-pane">
  <header class="pane-header">
    {#if breadcrumb.length > 0}
      <nav class="breadcrumb" aria-label="Path">
        {#each breadcrumb as seg, i (i)}
          {#if i > 0}<span class="crumb-sep">›</span>{/if}
          {#if i < breadcrumb.length - 1}
            <button
              class="crumb-link"
              type="button"
              onclick={() => onnavigate?.(seg.depth)}
              title="Navigate to {seg.label}"
            >{seg.label}</button>
          {:else}
            <span class="crumb-current">{seg.label}</span>
          {/if}
        {/each}
      </nav>
    {/if}
    <div class="title-row">
      {#if onback}
        <button class="back-btn" onclick={onback} title="Back (close editor)" disabled={saving}>‹</button>
      {/if}
      <span class="filepath">{filePath}</span>
      {#if dirty}<span class="dirty" title="Cambios sin guardar">●</span>{/if}
    </div>
    <button class="close-btn" onclick={cancel} title="Cerrar (Esc)" disabled={saving}>×</button>
  </header>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="pane-body"
  onclick={() => view?.focus()}
  role="presentation"
>
    {#if loading}
      <div class="state">Cargando…</div>
    {:else if error && !content}
      <div class="state error">{error}</div>
    {:else}
      <div bind:this={container} class="editor-host"></div>
    {/if}
  </div>

  {#if error && content}
    <p class="error-msg">{error}</p>
  {/if}

  <footer class="pane-footer">
    <span class="hint">
      {#if saved}
        <span class="saved-indicator">✓ Guardado</span>
      {:else}
        Ctrl+S guardar · Esc cerrar · Ctrl+F buscar · Ctrl+D seleccionar siguiente
      {/if}
    </span>
    <div class="actions">
      <Button variant="ghost" onclick={cancel} disabled={saving}>Cerrar</Button>
      <Button onclick={save} disabled={saving || loading || !dirty}>
        {saving ? 'Guardando…' : 'Guardar'}
      </Button>
    </div>
  </footer>
</div>

<style>
  .editor-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
    background: var(--color-surface);
    overflow: hidden;
    animation: editor-fade-in 0.22s cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes editor-fade-in {
    from {
      transform: translateX(8px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .pane-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1 1 auto;
    min-width: 0;
    font-size: var(--text-xs);
  }

  .crumb-link {
    background: none;
    border: none;
    padding: 2px 4px;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 3px;
    font-size: inherit;
  }

  .crumb-link:hover {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .crumb-current {
    color: var(--color-text);
    font-weight: 500;
    padding: 2px 4px;
  }

  .crumb-sep {
    color: var(--color-text-secondary);
    opacity: 0.5;
    user-select: none;
  }

  .back-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    color: var(--color-text-secondary);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .back-btn:hover:not(:disabled) {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .back-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .filepath {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .dirty {
    color: #ed8936;
    font-size: 18px;
    line-height: 1;
    animation: dirty-fade-in 0.18s ease-out;
  }

  @keyframes dirty-fade-in {
    from {
      opacity: 0;
      transform: scale(0.5);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 20px;
    line-height: 1;
    color: var(--color-text-secondary);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .close-btn:hover:not(:disabled) {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .close-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .pane-body {
    flex: 1;
    min-height: 0;
    display: flex;
    overflow: hidden;
    outline: none;
  }
  .pane-body:focus-visible {
    outline: 1px solid var(--color-accent);
    outline-offset: -1px;
  }

  .editor-host {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    display: flex;
  }

  .editor-host :global(.cm-editor) {
    flex: 1;
    width: 100%;
    height: 100%;
    min-height: 0;
  }

  .editor-host :global(.cm-scroller) {
    overflow: auto;
  }

  .state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
  }

  .state.error {
    color: #e53e3e;
    padding: 16px;
    text-align: center;
  }

  .error-msg {
    font-size: var(--text-sm);
    color: #e53e3e;
    margin: 0;
    padding: 8px 12px;
    background: rgba(229, 62, 62, 0.1);
    flex-shrink: 0;
  }

  .pane-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .hint {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  .saved-indicator {
    color: #48bb78;
    font-weight: 500;
    animation: saved-pulse 0.45s cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes saved-pulse {
    0% {
      opacity: 0;
      transform: scale(0.7);
    }
    40% {
      opacity: 1;
      transform: scale(1.15);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }

  .actions {
    display: flex;
    gap: 6px;
  }
</style>