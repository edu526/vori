<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { updatePreferences, detectTerminals, detectEditors } from '$lib/api/commands';
  import type { Preferences } from '$lib/api/types';

  const isOpen = $derived(dialogStore.current?.type === 'preferences');

  // Local editable copy of preferences
  let prefs = $state<Preferences>({
    default_editor: 'vscode',
    close_on_open: false,
    terminal: { available: {} },
    editors_available: {},
  });

  let detectError = $state('');
  let detecting = $state(false);
  let detectingEditors = $state(false);

  $effect(() => {
    if (!isOpen) return;
    // Deep copy to avoid mutating store directly
    prefs = JSON.parse(JSON.stringify(configStore.preferences));
    detectError = '';
  });

  async function handleDetectTerminals() {
    detecting = true;
    detectError = '';
    try {
      const available = await detectTerminals();
      prefs.terminal = {
        ...prefs.terminal,
        available,
        last_detected: new Date().toISOString(),
      };
    } catch (e) {
      detectError = String(e);
    } finally {
      detecting = false;
    }
  }

  async function handleSave() {
    try {
      await updatePreferences(prefs);
      configStore.preferences = prefs;
      dialogStore.close();
    } catch (e) {
      detectError = String(e);
    }
  }

  function handleBackdropKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') dialogStore.close();
  }

  function stopPropagation(e: MouseEvent | KeyboardEvent) {
    e.stopPropagation();
  }

  const terminalEntries = $derived(Object.entries(prefs.terminal.available ?? {}));
  const editorEntries = $derived(Object.entries(prefs.editors_available ?? {}).sort(([a], [b]) => a.localeCompare(b)));

  const EDITOR_LABELS: Record<string, string> = {
    'vscode': 'VSCode', 'vscode-insiders': 'VSCode Insiders', 'cursor': 'Cursor',
    'windsurf': 'Windsurf', 'kiro': 'Kiro', 'zed': 'Zed', 'fleet': 'Fleet',
    'sublime': 'Sublime Text', 'graviton': 'Graviton', 'helix': 'Helix',
    'neovim': 'Neovim', 'vim': 'Vim', 'emacs': 'Emacs', 'kate': 'Kate', 'gedit': 'Gedit',
  };

  async function handleDetectEditors() {
    detectingEditors = true;
    detectError = '';
    try {
      const available = await detectEditors();
      prefs.editors_available = available;
    } catch (e) {
      detectError = String(e);
    } finally {
      detectingEditors = false;
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => dialogStore.close()} onkeydown={handleBackdropKeydown}>
    <div class="dialog" role="dialog" aria-modal="true" tabindex="-1" onclick={stopPropagation} onkeydown={stopPropagation}>
      <div class="dialog-header">Preferences</div>

      <!-- Editor section -->
      <div class="section">
        <div class="section-title">Editor</div>

        <div class="field">
          <div class="field-label">Default editor</div>
          {#if editorEntries.length > 0}
            <div class="radio-group">
              {#each editorEntries as [key, execPath]}
                <label class="radio-label" title={execPath}>
                  <input
                    type="radio"
                    name="default_editor"
                    value={key}
                    bind:group={prefs.default_editor}
                  />
                  {EDITOR_LABELS[key] ?? key}
                </label>
              {/each}
            </div>
          {:else}
            <p class="hint">No editors detected yet.</p>
          {/if}
          <button class="btn btn-secondary detect-btn" onclick={handleDetectEditors} disabled={detectingEditors}>
            {detectingEditors ? 'Detecting...' : 'Detect Editors'}
          </button>
        </div>

        <div class="field">
          <label for="pref-text-editor">Text editor command</label>
          <input
            id="pref-text-editor"
            type="text"
            bind:value={prefs.default_text_editor}
            placeholder="xdg-open"
          />
        </div>
      </div>

      <!-- Behavior section -->
      <div class="section">
        <div class="section-title">Behavior</div>

        <div class="field">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={prefs.close_on_open} />
            Close launcher after opening
          </label>
        </div>
      </div>

      <!-- Terminal section -->
      <div class="section">
        <div class="section-title">Terminal</div>

        {#if terminalEntries.length > 0}
          <div class="field">
            <div class="field-label">Preferred terminal</div>
            <div class="radio-group">
              {#each terminalEntries as [name, execPath]}
                <label class="radio-label" title={execPath}>
                  <input
                    type="radio"
                    name="preferred_terminal"
                    value={name}
                    bind:group={prefs.terminal.preferred}
                  />
                  {name}
                </label>
              {/each}
            </div>
          </div>
        {:else}
          <p class="hint">No terminals detected yet.</p>
        {/if}

        {#if prefs.terminal.last_detected}
          <p class="hint">
            Last detected: {new Date(prefs.terminal.last_detected).toLocaleString()}
          </p>
        {/if}

        <button class="btn btn-secondary detect-btn" onclick={handleDetectTerminals} disabled={detecting}>
          {detecting ? 'Detecting...' : 'Detect Terminals'}
        </button>

        {#if detectError}
          <span class="error-msg">{detectError}</span>
        {/if}
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

  .section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 4px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field label,
  .field-label {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .field input {
    padding: 7px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.875rem;
  }

  .field input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.875rem;
    color: var(--color-text);
    cursor: pointer;
    font-weight: normal;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.875rem;
    color: var(--color-text);
    cursor: pointer;
    font-weight: normal;
  }

  .hint {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .detect-btn {
    align-self: flex-start;
  }

  .detect-btn:disabled {
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
