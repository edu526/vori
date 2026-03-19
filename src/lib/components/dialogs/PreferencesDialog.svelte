<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { updatePreferences, detectTerminals, detectEditors } from '$lib/api/commands';
  import type { Preferences } from '$lib/api/types';
  import AddEditorModal from './AddEditorModal.svelte';

  const isOpen = $derived(dialogStore.current?.type === 'preferences');

  let activeTab = $state<'appearance' | 'editors' | 'terminal'>('appearance');

  let prefs = $state<Preferences>({
    default_editor: 'vscode',
    close_on_open_editor: false,
    close_on_open_terminal: false,
    close_on_open_file: false,
    terminal: { available: {} },
    editors_available: {},
    theme: 'system',
  });

  let detectError = $state('');
  let detecting = $state(false);
  let detectingEditors = $state(false);
  let showAddEditor = $state(false);
  let showAddTerminal = $state(false);

  function handleAddEditor(name: string, exec: string) {
    const key = name.toLowerCase().replace(/\s+/g, '_');
    prefs.editors_available = { ...prefs.editors_available, [key]: exec };
    if (!prefs.default_editor) prefs.default_editor = key;
    showAddEditor = false;
  }

  function removeEditor(key: string) {
    const { [key]: _, ...rest } = prefs.editors_available;
    prefs.editors_available = rest;
    if (prefs.default_editor === key) {
      prefs.default_editor = Object.keys(rest)[0] ?? '';
    }
  }

  function handleAddTerminal(name: string, exec: string) {
    prefs.terminal = {
      ...prefs.terminal,
      available: { ...prefs.terminal.available, [name]: exec },
    };
    if (!prefs.terminal.preferred) prefs.terminal = { ...prefs.terminal, preferred: name };
    showAddTerminal = false;
  }

  function removeTerminal(name: string) {
    const { [name]: _, ...rest } = prefs.terminal.available;
    prefs.terminal = {
      ...prefs.terminal,
      available: rest,
      preferred: prefs.terminal.preferred === name ? Object.keys(rest)[0] : prefs.terminal.preferred,
    };
  }

  $effect(() => {
    if (!isOpen) return;
    prefs = JSON.parse(JSON.stringify(configStore.preferences));
    detectError = '';
    activeTab = 'appearance';
  });

  $effect(() => {
    if (showAddEditor) return dialogStore.pushEscape(() => (showAddEditor = false));
  });

  $effect(() => {
    if (showAddTerminal) return dialogStore.pushEscape(() => (showAddTerminal = false));
  });

  async function handleDetectTerminals() {
    detecting = true;
    detectError = '';
    try {
      const available = await detectTerminals();
      prefs.terminal = { ...prefs.terminal, available, last_detected: new Date().toISOString() };
    } catch (e) {
      detectError = String(e);
    } finally {
      detecting = false;
    }
  }

  async function handleDetectEditors() {
    detectingEditors = true;
    detectError = '';
    try {
      prefs.editors_available = await detectEditors();
    } catch (e) {
      detectError = String(e);
    } finally {
      detectingEditors = false;
    }
  }

  async function handleSave() {
    try {
      await updatePreferences(prefs);
      configStore.preferences = prefs;
      themeStore.apply(prefs.theme);
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
  const editorEntries = $derived(
    Object.entries(prefs.editors_available ?? {}).sort(([a], [b]) => a.localeCompare(b))
  );

  const EDITOR_LABELS: Record<string, string> = {
    vscode: 'VSCode', 'vscode-insiders': 'VSCode Insiders', cursor: 'Cursor',
    windsurf: 'Windsurf', kiro: 'Kiro', zed: 'Zed', fleet: 'Fleet',
    sublime: 'Sublime Text', graviton: 'Graviton', helix: 'Helix',
    neovim: 'Neovim', vim: 'Vim', emacs: 'Emacs', kate: 'Kate', gedit: 'Gedit',
    antigravity: 'Antigravity',
  };
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => dialogStore.close()} onkeydown={handleBackdropKeydown}>
    <div class="dialog" role="dialog" aria-modal="true" tabindex="-1" onclick={stopPropagation} onkeydown={stopPropagation}>

      <div class="dialog-header">Preferences</div>

      <!-- Tabs -->
      <div class="tabs">
        <button class="tab" class:active={activeTab === 'appearance'} onclick={() => activeTab = 'appearance'}>
          Appearance
        </button>
        <button class="tab" class:active={activeTab === 'editors'} onclick={() => activeTab = 'editors'}>
          Editors
        </button>
        <button class="tab" class:active={activeTab === 'terminal'} onclick={() => activeTab = 'terminal'}>
          Terminal
        </button>
      </div>

      <!-- Tab: Appearance -->
      {#if activeTab === 'appearance'}
        <div class="tab-content">
          <div class="field">
            <div class="field-label">Theme</div>
            <div class="theme-options">
              <label class="theme-option" class:active={prefs.theme === 'light'}>
                <input type="radio" name="theme" value="light" bind:group={prefs.theme} />
                <div class="theme-preview theme-preview--light">
                  <div class="tp-sidebar"></div>
                  <div class="tp-content">
                    <div class="tp-bar"></div>
                    <div class="tp-bar tp-bar--short"></div>
                  </div>
                </div>
                <span>Light</span>
              </label>

              <label class="theme-option" class:active={prefs.theme === 'dark'}>
                <input type="radio" name="theme" value="dark" bind:group={prefs.theme} />
                <div class="theme-preview theme-preview--dark">
                  <div class="tp-sidebar"></div>
                  <div class="tp-content">
                    <div class="tp-bar"></div>
                    <div class="tp-bar tp-bar--short"></div>
                  </div>
                </div>
                <span>Dark</span>
              </label>

              <label class="theme-option" class:active={prefs.theme === 'system'}>
                <input type="radio" name="theme" value="system" bind:group={prefs.theme} />
                <div class="theme-preview theme-preview--system">
                  <div class="tp-half tp-half--light"><div class="tp-sidebar"></div></div>
                  <div class="tp-half tp-half--dark"><div class="tp-sidebar"></div></div>
                </div>
                <span>System</span>
              </label>
            </div>
          </div>
        </div>
      {/if}

      <!-- Tab: Editors -->
      {#if activeTab === 'editors'}
        <div class="tab-content">
          <div class="field">
            <div class="field-label">Default editor for projects</div>
            {#if editorEntries.length > 0}
              <div class="radio-group">
                {#each editorEntries as [key, execPath]}
                  <div class="radio-row">
                    <label class="radio-label" title={execPath}>
                      <input type="radio" name="default_editor" value={key} bind:group={prefs.default_editor} />
                      {EDITOR_LABELS[key] ?? key}
                    </label>
                    <button class="remove-btn" onclick={() => removeEditor(key)} title="Remove">✕</button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="hint">No editors detected yet.</p>
            {/if}
            <div class="detect-row">
              <button class="btn btn-secondary detect-btn" onclick={handleDetectEditors} disabled={detectingEditors}>
                {#if detectingEditors}<span class="spinner"></span>{/if}
                {detectingEditors ? 'Detecting...' : 'Detect Editors'}
              </button>
              <button class="btn btn-secondary" onclick={() => (showAddEditor = true)}>
                + Add Editor
              </button>
            </div>
          </div>

          <div class="field">
            <label for="pref-text-editor">Text file editor command</label>
            <div class="input-row">
              <input
                id="pref-text-editor"
                type="text"
                bind:value={prefs.default_text_editor}
                placeholder="xdg-open"
              />
              <button class="btn btn-secondary browse-btn" onclick={async () => {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const sel = await open({ multiple: false, title: 'Select text editor binary' });
                if (sel) prefs.default_text_editor = typeof sel === 'string' ? sel : sel[0];
              }}>Browse</button>
            </div>
          </div>

          <div class="divider"></div>

          <div class="field">
            <div class="field-label">After opening</div>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={prefs.close_on_open_editor} />
              Close Vori after opening a project in editor
            </label>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={prefs.close_on_open_file} />
              Close Vori after opening a text file
            </label>
          </div>
        </div>
      {/if}

      <!-- Tab: Terminal -->
      {#if activeTab === 'terminal'}
        <div class="tab-content">
          <div class="field">
            <div class="field-label">Preferred terminal</div>
            {#if terminalEntries.length > 0}
              <div class="radio-group">
                {#each terminalEntries as [name, execPath]}
                  <div class="radio-row">
                    <label class="radio-label" title={execPath}>
                      <input type="radio" name="preferred_terminal" value={name} bind:group={prefs.terminal.preferred} />
                      {name}
                    </label>
                    <button class="remove-btn" onclick={() => removeTerminal(name)} title="Remove">✕</button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="hint">No terminals detected yet.</p>
            {/if}
            {#if prefs.terminal.last_detected}
              <p class="hint">Last detected: {new Date(prefs.terminal.last_detected).toLocaleString()}</p>
            {/if}
            <div class="detect-row">
              <button class="btn btn-secondary detect-btn" onclick={handleDetectTerminals} disabled={detecting}>
                {#if detecting}<span class="spinner"></span>{/if}
                {detecting ? 'Detecting...' : 'Detect Terminals'}
              </button>
              <button class="btn btn-secondary" onclick={() => (showAddTerminal = true)}>
                + Add Terminal
              </button>
            </div>
          </div>

          <div class="divider"></div>

          <div class="field">
            <div class="field-label">After opening</div>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={prefs.close_on_open_terminal} />
              Close Vori after opening a terminal
            </label>
          </div>

          {#if detectError}
            <span class="error-msg">{detectError}</span>
          {/if}
        </div>
      {/if}

      <div class="dialog-actions">
        <button class="btn btn-secondary" onclick={() => dialogStore.close()}>Cancel</button>
        <button class="btn btn-primary" onclick={handleSave}>Save</button>
      </div>
    </div>
  </div>

  {#if showAddEditor}
    <AddEditorModal title="Add Editor" onAdd={handleAddEditor} onClose={() => (showAddEditor = false)} />
  {/if}

  {#if showAddTerminal}
    <AddEditorModal title="Add Terminal" onAdd={handleAddTerminal} onClose={() => (showAddTerminal = false)} />
  {/if}
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
    padding: 20px;
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

  /* Tabs */
  .tabs {
    display: flex;
    gap: 2px;
    background: var(--color-bg);
    border-radius: 8px;
    padding: 3px;
  }

  .tab {
    flex: 1;
    padding: 6px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .tab.active {
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }

  /* Tab content */
  .tab-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
    height: 260px;
    overflow-y: auto;
  }

  .divider {
    height: 1px;
    background: var(--color-border);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field label,
  .field-label {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .field input[type='text'] {
    padding: 7px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.875rem;
  }

  .field input[type='text']:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .radio-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
  }

  .radio-label,
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.875rem;
    color: var(--color-text);
    cursor: pointer;
    font-weight: normal;
    flex: 1;
  }

  .remove-btn {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.7rem;
    padding: 2px 5px;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.15s, background 0.15s, color 0.15s;
  }

  .radio-row:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    background: #fee2e2;
    color: #dc2626;
  }

  .input-row {
    display: flex;
    gap: 6px;
  }

  .input-row input {
    flex: 1;
  }

  .browse-btn {
    white-space: nowrap;
    flex-shrink: 0;
  }

  .hint {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .detect-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .detect-btn {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .detect-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding-top: 4px;
    border-top: 1px solid var(--color-border);
  }

  .btn {
    padding: 7px 16px;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: opacity 0.15s, filter 0.15s;
  }

  .btn:hover:not(:disabled) {
    filter: brightness(0.92);
  }

  .btn:active:not(:disabled) {
    filter: brightness(0.85);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
  }

  .btn-secondary {
    background: var(--color-hover);
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }

  .error-msg {
    color: #e53e3e;
    font-size: 0.8rem;
  }

  /* Theme picker */
  .theme-options {
    display: flex;
    gap: 8px;
  }

  .theme-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 8px;
    border: 2px solid var(--color-border);
    border-radius: 8px;
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-weight: normal;
    background: var(--color-bg);
    transition: border-color 0.15s;
  }

  .theme-option.active {
    border-color: var(--color-accent);
    color: var(--color-accent);
    font-weight: 500;
  }

  .theme-option input { display: none; }

  .theme-preview {
    width: 100%;
    height: 52px;
    border-radius: 4px;
    overflow: hidden;
    display: flex;
    border: 1px solid rgba(0, 0, 0, 0.08);
  }

  .theme-preview--light { background: #f0f2f5; }
  .theme-preview--dark  { background: #191b2f; }

  .theme-preview--light .tp-sidebar { background: #e4e7f0; width: 30%; }
  .theme-preview--dark  .tp-sidebar { background: #252845; width: 30%; }

  .tp-content {
    flex: 1;
    padding: 6px 5px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tp-bar {
    height: 5px;
    border-radius: 2px;
    background: #007c61;
    width: 80%;
  }

  .tp-bar--short { width: 50%; opacity: 0.4; }

  .theme-preview--system { background: transparent; }

  .tp-half { flex: 1; display: flex; }
  .tp-half--light { background: #f0f2f5; }
  .tp-half--dark  { background: #191b2f; }

  .tp-half--light .tp-sidebar { background: #e4e7f0; width: 40%; }
  .tp-half--dark  .tp-sidebar { background: #252845; width: 40%; }
</style>
