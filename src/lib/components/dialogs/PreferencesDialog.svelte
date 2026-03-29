<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { updatePreferences, detectTerminals, detectEditors } from '$lib/api/commands';
  import { type } from '@tauri-apps/plugin-os';
  import type { Preferences } from '$lib/api/types';
  import AddEditorModal from './AddEditorModal.svelte';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Tabs, TabsList, TabsTrigger, TabsContent } from '$lib/components/ui/tabs';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';

  const isOpen = $derived(dialogStore.current?.type === 'preferences');

  let activeTab = $state<'appearance' | 'editors' | 'terminal' | 'system'>('appearance');

  let prefs = $state<Preferences>({
    default_editor: 'vscode',
    close_on_open_editor: false,
    close_on_open_terminal: false,
    close_on_open_file: false,
    terminal: { available: {} },
    editors_available: {},
    theme: 'system',
    autostart: true,
    show_tray: true,
    keep_background: true,
    hotkey: 'Super+Shift+KeyV',
    ui_scale: 1.0,
  });

  let recordingHotkey = $state(false);
  let hotkeyError = $state('');
  let osType = $state('');
  let originalScale = $state(1.0);
  
  $effect(() => {
    osType = type();
  });

  function startRecording() {
    recordingHotkey = true;
    hotkeyError = '';
  }

  function stopRecording() {
    recordingHotkey = false;
  }

  function handleHotkeyKeydown(e: KeyboardEvent) {
    if (!recordingHotkey) return;
    
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      recordingHotkey = false;
      hotkeyError = '';
      return;
    }

    e.preventDefault();
    e.stopPropagation();

    // Ignore lone modifier presses
    if (['Meta', 'Control', 'Alt', 'Shift'].includes(e.key)) return;

    if (e.key === 'Backspace' || e.key === 'Delete') {
      prefs.hotkey = '';
      hotkeyError = '';
      recordingHotkey = false;
      return;
    }

    const parts: string[] = [];
    if (e.metaKey) parts.push('Super');
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');

    // Convert event.code to Tauri shortcut key name
    // e.g. "KeyV" → "KeyV", "Space" → "Space", "F1" → "F1"
    const code = e.code; // already in the right format for Tauri
    parts.push(code);

    if (parts.length < 2) {
      hotkeyError = 'Use at least one modifier (Super, Ctrl, Alt, Shift)';
      return;
    }

    prefs.hotkey = parts.join('+');
    hotkeyError = '';
    recordingHotkey = false;
  }

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
    if (prefs.default_editor === key) prefs.default_editor = Object.keys(rest)[0] ?? '';
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
    originalScale = configStore.preferences.ui_scale ?? 1.0;
    detectError = '';
    activeTab = 'appearance';
  });

  // Live preview while dialog is open
  $effect(() => {
    if (!isOpen) return;
    void prefs.ui_scale; // track reactively
    themeStore.applyScale(prefs.ui_scale ?? 1.0);
  });

  function adjustScale(delta: number) {
    const current = Math.round((prefs.ui_scale ?? 1.0) * 10) / 10;
    prefs.ui_scale = Math.max(0.8, Math.min(1.5, Math.round((current + delta) * 10) / 10));
  }

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
      originalScale = prefs.ui_scale ?? 1.0; // mark as saved so close doesn't revert
      dialogStore.close();
    } catch (e) {
      detectError = String(e);
    }
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


<Dialog open={isOpen} onOpenChange={(o) => { if (!o) { themeStore.applyScale(originalScale); dialogStore.close(); } }}>
  <DialogContent 
    class="w-[440px] max-w-[90vw] h-[460px] flex flex-col gap-0 p-0 overflow-hidden" 
    showCloseButton={false}
    escapeKeydownBehavior={recordingHotkey ? 'ignore' : 'close'}
    onEscapeKeydown={(e) => { 
      if (recordingHotkey) {
        e.preventDefault();
        recordingHotkey = false;
        hotkeyError = '';
      }
    }}
  >
    <DialogHeader class="px-6 pt-5 pb-3">
      <DialogTitle>Preferences</DialogTitle>
    </DialogHeader>

    <Tabs value={activeTab} onValueChange={(v) => (activeTab = v as typeof activeTab)} class="flex flex-col flex-1 min-h-0">
      <TabsList class="mx-6 shrink-0">
        <TabsTrigger value="appearance">Appearance</TabsTrigger>
        <TabsTrigger value="editors">Editors</TabsTrigger>
        <TabsTrigger value="terminal">Terminal</TabsTrigger>
        <TabsTrigger value="system">System</TabsTrigger>
      </TabsList>

      <!-- Appearance -->
      <TabsContent value="appearance" class="tab-body">
        <div class="field">
          <Label>Theme</Label>
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

        <div class="field">
          <Label>Zoom</Label>
          <div class="flex items-center gap-1">
            <Button
              variant="outline"
              size="icon"
              onclick={() => adjustScale(-0.1)}
              disabled={(prefs.ui_scale ?? 1.0) <= 0.8}
              aria-label="Decrease scale"
            >−</Button>
            <Button
              variant="outline"
              size="sm"
              class="min-w-[58px] font-mono"
              onclick={() => (prefs.ui_scale = 1.0)}
              title="Reset to 100%"
            >{Math.round((prefs.ui_scale ?? 1.0) * 100)}%</Button>
            <Button
              variant="outline"
              size="icon"
              onclick={() => adjustScale(0.1)}
              disabled={(prefs.ui_scale ?? 1.0) >= 1.5}
              aria-label="Increase scale"
            >+</Button>
          </div>
        </div>
      </TabsContent>

      <!-- Editors -->
      <TabsContent value="editors" class="tab-body">
        <div class="field">
          <Label>Default editor for projects</Label>
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
            <Button variant="outline" size="sm" onclick={handleDetectEditors} disabled={detectingEditors}>
              {#if detectingEditors}<span class="spinner"></span>{/if}
              {detectingEditors ? 'Detecting...' : 'Detect Editors'}
            </Button>
            <Button variant="outline" size="sm" onclick={() => (showAddEditor = true)}>
              + Add Editor
            </Button>
          </div>
        </div>

        <div class="field">
          <Label for="pref-text-editor">Text file editor command</Label>
          <div class="input-row">
            <Input id="pref-text-editor" bind:value={prefs.default_text_editor} placeholder="xdg-open" />
            <Button variant="outline" size="sm" onclick={async () => {
              const { open } = await import('@tauri-apps/plugin-dialog');
              const sel = await open({ multiple: false, title: 'Select text editor binary' });
              if (sel) prefs.default_text_editor = typeof sel === 'string' ? sel : sel[0];
            }}>Browse</Button>
          </div>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>After opening</Label>
          <label class="check-row">
            <Checkbox bind:checked={prefs.close_on_open_editor} />
            <span>Close Vori after opening a project in editor</span>
          </label>
          <label class="check-row">
            <Checkbox bind:checked={prefs.close_on_open_file} />
            <span>Close Vori after opening a text file</span>
          </label>
        </div>
      </TabsContent>

      <!-- Terminal -->
      <TabsContent value="terminal" class="tab-body">
        <div class="field">
          <Label>Preferred terminal</Label>
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
            <Button variant="outline" size="sm" onclick={handleDetectTerminals} disabled={detecting}>
              {#if detecting}<span class="spinner"></span>{/if}
              {detecting ? 'Detecting...' : 'Detect Terminals'}
            </Button>
            <Button variant="outline" size="sm" onclick={() => (showAddTerminal = true)}>
              + Add Terminal
            </Button>
          </div>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>After opening</Label>
          <label class="check-row">
            <Checkbox bind:checked={prefs.close_on_open_terminal} />
            <span>Close Vori after opening a terminal</span>
          </label>
        </div>

        {#if detectError}
          <p class="error-msg">{detectError}</p>
        {/if}
      </TabsContent>

      <!-- System -->
      <TabsContent value="system" class="tab-body">
        <div class="field">
          <Label>Startup & Background Behavior</Label>
          <label class="check-row flex items-start gap-2">
            <Checkbox bind:checked={prefs.autostart} class="mt-1" />
            <div class="flex flex-col">
              <span>Launch automatically at login</span>
              <span class="text-[0.78rem] text-muted-foreground">Start Vori silently in the background when your computer boots.</span>
            </div>
          </label>
          <label class="check-row flex items-start gap-2">
            <Checkbox bind:checked={prefs.keep_background} class="mt-1" />
            <div class="flex flex-col">
              <span>Keep running in background</span>
              <span class="text-[0.78rem] text-muted-foreground">Closing the window hides it instead of quitting. Highly recommended so Vori stays instantly available.</span>
            </div>
          </label>
          <label class="check-row flex items-start gap-2">
            <Checkbox bind:checked={prefs.show_tray} class="mt-1" />
            <div class="flex flex-col">
              <span>Show icon in System Tray</span>
              <span class="text-[0.78rem] text-muted-foreground">Provides an icon to easily summon or quit Vori.</span>
            </div>
          </label>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>Global shortcut</Label>
          <p class="hint">Press this combination anywhere to show or hide Vori.</p>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div style="display: flex; gap: 8px; align-items: stretch;">
            <div
              class="hotkey-recorder"
              style="flex: 1;"
              class:recording={recordingHotkey}
              tabindex="0"
              role="button"
              aria-label="Click to record shortcut"
              onclick={startRecording}
              onblur={stopRecording}
              onkeydown={handleHotkeyKeydown}
            >
              {#if recordingHotkey}
                <span class="hotkey-recording-label">Press your shortcut (Esc to cancel, Backspace to clear)…</span>
              {:else}
                <kbd class="hotkey-display">{prefs.hotkey || 'Not set'}</kbd>
                <span class="hotkey-hint">Click to change</span>
              {/if}
            </div>
            {#if prefs.hotkey && !recordingHotkey}
              <Button variant="outline" onclick={(e) => { e.stopPropagation(); prefs.hotkey = ''; hotkeyError = ''; }}>Disable</Button>
            {/if}
          </div>
          {#if hotkeyError}
            <p class="error-msg">{hotkeyError}</p>
          {/if}
          <p class="hint">Format: <code>Super+Shift+KeyV</code> — modifiers: Super, Ctrl, Alt, Shift</p>
          {#if osType === 'linux'}
            <p class="hint" style="margin-top: 4px; color: #a1a1aa;">
              <strong>Note for Linux/Wayland users:</strong> Due to OS security, this may only work when Vori has focus. 
              For a true global hotkey, disable this and add a custom shortcut in your system settings to run the <code>vori</code> command.
            </p>
          {/if}
        </div>
      </TabsContent>
    </Tabs>

    <DialogFooter class="px-6 pb-5 pt-3 border-t border-border mt-2">
      <Button variant="ghost" onclick={() => dialogStore.close()}>Cancel</Button>
      <Button onclick={handleSave}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

{#if showAddEditor}
  <AddEditorModal title="Add Editor" onAdd={handleAddEditor} onClose={() => (showAddEditor = false)} />
{/if}

{#if showAddTerminal}
  <AddEditorModal title="Add Terminal" onAdd={handleAddTerminal} onClose={() => (showAddTerminal = false)} />
{/if}

<style>
  /* Force TabsList background with Vori variables as fallback */
  :global([data-slot="tabs-list"]) {
    background: var(--color-hover);
  }

  :global(.tab-body) {
    display: flex;
    flex-direction: column;
    gap: 14px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 12px 24px 4px;
  }

  .divider { height: 1px; background: var(--color-border); }

  .field { display: flex; flex-direction: column; gap: 6px; }

  .radio-group { display: flex; flex-direction: column; gap: 6px; }

  .radio-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-base);
    color: var(--color-text);
    cursor: pointer;
    flex: 1;
  }

  .check-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--text-base);
    color: var(--color-text);
    cursor: pointer;
  }

  .remove-btn {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: var(--text-2xs);
    padding: 2px 5px;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.15s, background 0.15s, color 0.15s;
  }

  .radio-row:hover .remove-btn { opacity: 1; }
  .remove-btn:hover { background: #fee2e2; color: #dc2626; }

  .input-row { display: flex; gap: 6px; }
  .input-row :global(input) { flex: 1; }

  .detect-row { display: flex; gap: 8px; flex-wrap: wrap; }

  .hint { font-size: var(--text-sm); color: var(--color-text-secondary); margin: 0; }
  .error-msg { color: #e53e3e; font-size: var(--text-sm); }

  .spinner {
    width: 12px; height: 12px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    flex-shrink: 0;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Hotkey recorder */
  .hotkey-recorder {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border: 1.5px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    cursor: pointer;
    min-height: 38px;
    outline: none;
    transition: border-color 0.15s;
  }
  .hotkey-recorder:hover,
  .hotkey-recorder:focus { border-color: var(--color-accent); }
  .hotkey-recorder.recording {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-bg));
  }

  .hotkey-display {
    font-family: ui-monospace, monospace;
    font-size: var(--text-sm);
    background: var(--color-hover);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    padding: 2px 7px;
    color: var(--color-text);
  }
  .hotkey-hint {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }
  .hotkey-recording-label {
    font-size: var(--text-sm);
    color: var(--color-accent);
    font-style: italic;
  }

  /* Theme picker */
  .theme-options { display: flex; gap: 8px; }

  .theme-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 8px;
    border: 2px solid var(--color-border);
    border-radius: 8px;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
    background: var(--color-bg);
    transition: border-color 0.15s;
  }
  .theme-option.active { border-color: var(--color-accent); color: var(--color-accent); font-weight: 500; }
  .theme-option input { display: none; }

  .theme-preview {
    width: 100%; height: 52px;
    border-radius: 4px; overflow: hidden;
    display: flex;
    border: 1px solid rgba(0,0,0,0.08);
  }
  .theme-preview--light { background: #f0f2f5; }
  .theme-preview--dark  { background: #191b2f; }
  .theme-preview--light .tp-sidebar { background: #e4e7f0; width: 30%; }
  .theme-preview--dark  .tp-sidebar { background: #252845; width: 30%; }
  .tp-content { flex: 1; padding: 6px 5px; display: flex; flex-direction: column; gap: 4px; }
  .tp-bar { height: 5px; border-radius: 2px; background: #007c61; width: 80%; }
  .tp-bar--short { width: 50%; opacity: 0.4; }
  .theme-preview--system { background: transparent; }
  .tp-half { flex: 1; display: flex; }
  .tp-half--light { background: #f0f2f5; }
  .tp-half--dark  { background: #191b2f; }
  .tp-half--light .tp-sidebar { background: #e4e7f0; width: 40%; }
  .tp-half--dark  .tp-sidebar { background: #252845; width: 40%; }

</style>
