<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { updatePreferences, detectTerminals, detectEditors, exportConfig, importConfig } from '$lib/api/commands';
  import { updaterStore } from '$lib/stores/updater.svelte';
  import { t, i18n } from '$lib/i18n/index.svelte';
  import { open, save, ask, message } from '@tauri-apps/plugin-dialog';
  import type { Preferences } from '$lib/api/types';
  import AddEditorModal from './AddEditorModal.svelte';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Tabs, TabsList, TabsTrigger, TabsContent } from '$lib/components/ui/tabs';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';

  const isOpen = $derived(dialogStore.current?.type === 'preferences');

  let activeTab = $state<'appearance' | 'editors' | 'terminal' | 'claude' | 'system'>('appearance');

  let prefs = $state<Preferences>({
    default_editor: 'vscode',
    close_on_open_editor: false,
    close_on_open_terminal: false,
    close_on_open_file: false,
    terminal: { available: {} },
    editors_available: {},
    theme: 'system',
    language: 'system',
    autostart: true,
    show_tray: true,
    keep_background: true,
    hotkey: 'Super+Shift+KeyV',
    ui_scale: 1.0,
    editor_text_wrap: false,
    editor_tab_size: 2,
    editor_font_size: 13,
    claude_profiles: {},
  });

  let recordingHotkey = $state(false);
  let hotkeyError = $state('');
  let osType = $state('');
  let originalScale = $state(1.0);
  
  $effect(() => {
    import('@tauri-apps/plugin-os').then(({ type }) => {
      osType = type();
    });
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

    prefs.hotkey = parts.join('+');
    hotkeyError = '';
    recordingHotkey = false;
  }

  let detectError = $state('');
  let detecting = $state(false);
  let detectingEditors = $state(false);
  let showAddEditor = $state(false);
  let showAddTerminal = $state(false);

  // ── Backup ──────────────────────────────────────────────────────────────────
  let backupBusy = $state(false);

  async function handleExport() {
    if (backupBusy) return;
    const stamp = new Date().toISOString().slice(0, 10);
    const target = await save({
      title: t('backup.exportTitle'),
      defaultPath: `vori-backup-${stamp}.json`,
      filters: [{ name: t('backup.filter'), extensions: ['json'] }],
    });
    if (!target) return;
    backupBusy = true;
    try {
      await exportConfig(target);
      await message(t('backup.saved', { path: target }), { title: t('backup.exportDone'), kind: 'info' });
    } catch (e) {
      await message(String(e), { title: t('backup.exportFailed'), kind: 'error' });
    } finally {
      backupBusy = false;
    }
  }

  async function handleImport() {
    if (backupBusy) return;
    const picked = await open({
      title: t('backup.importTitle'),
      multiple: false,
      filters: [{ name: t('backup.filter'), extensions: ['json'] }],
    });
    if (typeof picked !== 'string') return;
    const ok = await ask(
      t('backup.importConfirm'),
      { title: t('backup.importTitle'), kind: 'warning', okLabel: t('backup.importAction'), cancelLabel: t('common.cancel') },
    );
    if (!ok) return;
    backupBusy = true;
    try {
      const summary = await importConfig(picked);
      await configStore.load();
      themeStore.apply(configStore.preferences.theme ?? 'system');
      i18n.setPreference(configStore.preferences.language ?? 'system');
      themeStore.applyScale(configStore.preferences.ui_scale ?? 1.0);
      navigationStore.refresh(
        configStore.categories, configStore.projects,
        configStore.files, configStore.favorites, configStore.recents,
      );
      dialogStore.close();
      await message(
        t('backup.importSummary', {
          categories: summary.categories,
          projects: summary.projects,
          files: summary.files,
          copy: summary.safety_copy,
        }),
        { title: t('backup.importDone'), kind: 'info' },
      );
    } catch (e) {
      await message(String(e), { title: t('backup.importFailed'), kind: 'error' });
    } finally {
      backupBusy = false;
    }
  }

  async function handleCheckForUpdate() {
    if (updaterStore.state === 'checking') return;
    await updaterStore.refresh();
    if (updaterStore.state === 'available') {
      await updaterStore.promptInstall();
    } else if (updaterStore.state === 'up-to-date') {
      await message(t('update.upToDate', { version: __APP_VERSION__ }), {
        title: t('update.upToDateTitle'),
        kind: 'info',
      });
    } else if (updaterStore.state === 'error') {
      await message(t('update.checkFailed', { error: updaterStore.lastError }), {
        title: t('update.checkFailedTitle'),
        kind: 'error',
      });
    }
  }

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

  let newProfileName = $state('');
  let newProfileDir = $state('');
  let profileError = $state('');

  function addProfile() {
    const name = newProfileName.trim();
    const dir = newProfileDir.trim();
    if (!name) { profileError = t('dialog.nameRequired'); return; }
    if (!dir) { profileError = t('prefs.profileDirRequired'); return; }
    if (name in prefs.claude_profiles) { profileError = t('prefs.profileExists', { name }); return; }
    prefs.claude_profiles = { ...prefs.claude_profiles, [name]: dir };
    newProfileName = '';
    newProfileDir = '';
    profileError = '';
  }

  function removeProfile(name: string) {
    const { [name]: _, ...rest } = prefs.claude_profiles;
    prefs.claude_profiles = rest;
  }

  async function browseProfileDir() {
    const picked = await open({ directory: true });
    if (typeof picked === 'string') newProfileDir = picked;
  }

  $effect(() => {
    if (!isOpen) return;
    prefs = JSON.parse(JSON.stringify(configStore.preferences));
    originalScale = configStore.preferences.ui_scale ?? 1.0;
    detectError = '';
    newProfileName = '';
    newProfileDir = '';
    profileError = '';
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
    // Sync keep_background and show_tray — the UI now exposes them as a
    // single "When closing the window" dropdown, so they must always match.
    // Done here (not on load) to avoid a reactive loop in the load $effect.
    prefs.keep_background = prefs.show_tray;
    // Removing a profile makes the backend clear it from categories/projects
    const removedProfile = Object.keys(configStore.preferences.claude_profiles ?? {})
      .some((n) => !(n in prefs.claude_profiles));
    try {
      await updatePreferences(prefs);
      configStore.preferences = prefs;
      if (removedProfile) {
        await configStore.load();
        navigationStore.refresh(
          configStore.categories, configStore.projects,
          configStore.files, configStore.favorites, configStore.recents,
        );
      }
      themeStore.apply(prefs.theme);
      i18n.setPreference(prefs.language);
      originalScale = prefs.ui_scale ?? 1.0; // mark as saved so close doesn't revert
      dialogStore.close();
    } catch (e) {
      detectError = String(e);
    }
  }

  const profileEntries = $derived(
    Object.entries(prefs.claude_profiles ?? {}).sort(([a], [b]) => a.localeCompare(b)),
  );
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
      <DialogTitle>{t('toolbar.preferences')}</DialogTitle>
    </DialogHeader>

    <Tabs value={activeTab} onValueChange={(v) => (activeTab = v as typeof activeTab)} class="flex flex-col flex-1 min-h-0">
      <TabsList class="mx-6 shrink-0">
        <TabsTrigger value="appearance">{t('prefs.tab.appearance')}</TabsTrigger>
        <TabsTrigger value="editors">{t('prefs.tab.editors')}</TabsTrigger>
        <TabsTrigger value="terminal">{t('common.terminal')}</TabsTrigger>
        <TabsTrigger value="claude">{t('prefs.tab.claude')}</TabsTrigger>
        <TabsTrigger value="system">{t('prefs.tab.system')}</TabsTrigger>
      </TabsList>

      <!-- Appearance -->
      <TabsContent value="appearance" class="tab-body">
        <div class="field">
          <Label>{t('prefs.theme')}</Label>
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
              <span>{t('prefs.theme.light')}</span>
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
              <span>{t('prefs.theme.dark')}</span>
            </label>
            <label class="theme-option" class:active={prefs.theme === 'system'}>
              <input type="radio" name="theme" value="system" bind:group={prefs.theme} />
              <div class="theme-preview theme-preview--system">
                <div class="tp-half tp-half--light"><div class="tp-sidebar"></div></div>
                <div class="tp-half tp-half--dark"><div class="tp-sidebar"></div></div>
              </div>
              <span>{t('prefs.theme.system')}</span>
            </label>
          </div>
        </div>

        <div class="field">
          <Label for="pref-language">{t('lang.label')}</Label>
          <select id="pref-language" class="native-select" bind:value={prefs.language}>
            <option value="system">{t('lang.system')}</option>
            <option value="en">{t('lang.en')}</option>
            <option value="es">{t('lang.es')}</option>
          </select>
        </div>

        <div class="field">
          <Label>Zoom</Label>
          <div class="flex items-center gap-1">
            <Button
              variant="outline"
              size="icon"
              onclick={() => adjustScale(-0.1)}
              disabled={(prefs.ui_scale ?? 1.0) <= 0.8}
              aria-label={t('prefs.zoomOut')}
            >−</Button>
            <Button
              variant="outline"
              size="sm"
              class="min-w-[58px] font-mono"
              onclick={() => (prefs.ui_scale = 1.0)}
              title={t('prefs.zoomReset')}
            >{Math.round((prefs.ui_scale ?? 1.0) * 100)}%</Button>
            <Button
              variant="outline"
              size="icon"
              onclick={() => adjustScale(0.1)}
              disabled={(prefs.ui_scale ?? 1.0) >= 1.5}
              aria-label={t('prefs.zoomIn')}
            >+</Button>
          </div>
        </div>
      </TabsContent>

      <!-- Editors -->
      <TabsContent value="editors" class="tab-body">
        <div class="field">
          <Label>{t('prefs.defaultEditor')}</Label>
          {#if editorEntries.length > 0}
            <div class="radio-group">
              {#each editorEntries as [key, execPath]}
                <div class="radio-row">
                  <label class="radio-label" title={execPath}>
                    <input type="radio" name="default_editor" value={key} bind:group={prefs.default_editor} />
                    {EDITOR_LABELS[key] ?? key}
                  </label>
                  <button class="remove-btn" onclick={() => removeEditor(key)} title={t('common.remove')}>✕</button>
                </div>
              {/each}
            </div>
          {:else}
            <p class="hint">{t('prefs.noEditors')}</p>
          {/if}
          <div class="detect-row">
            <Button variant="outline" size="sm" onclick={handleDetectEditors} disabled={detectingEditors}>
              {#if detectingEditors}<span class="spinner"></span>{/if}
              {detectingEditors ? t('prefs.detecting') : t('prefs.detectEditors')}
            </Button>
            <Button variant="outline" size="sm" onclick={() => (showAddEditor = true)}>
              {t('prefs.addEditor')}
            </Button>
          </div>
        </div>

        <div class="field">
          <Label for="pref-text-editor">{t('prefs.textEditorCmd')}</Label>
          <div class="input-row">
            <Input id="pref-text-editor" bind:value={prefs.default_text_editor} placeholder="xdg-open" />
            <Button variant="outline" size="sm" onclick={async () => {
              const sel = await open({ multiple: false, title: t('prefs.selectTextEditor') });
              if (sel) prefs.default_text_editor = typeof sel === 'string' ? sel : sel[0];
            }}>{t('common.browse')}</Button>
          </div>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>{t('prefs.inAppEditor')}</Label>
          <label class="check-row">
            <Checkbox bind:checked={prefs.editor_text_wrap} />
            <span>{t('prefs.wrapLines')}</span>
          </label>
          <div class="pref-row">
            <Label for="pref-tab-size">{t('prefs.tabSize')}</Label>
            <select
              id="pref-tab-size"
              class="native-select"
              value={prefs.editor_tab_size}
              onchange={(e) => {
                prefs.editor_tab_size = Number((e.currentTarget as HTMLSelectElement).value);
              }}
            >
              <option value={2}>{t('prefs.spaces', { count: 2 })}</option>
              <option value={4}>{t('prefs.spaces', { count: 4 })}</option>
            </select>
          </div>
          <div class="pref-row">
            <Label for="pref-font-size">{t('prefs.fontSize')}</Label>
            <div class="flex items-center gap-1">
              <Button
                variant="outline"
                size="icon"
                onclick={() => prefs.editor_font_size = Math.max(11, prefs.editor_font_size - 1)}
                disabled={prefs.editor_font_size <= 11}
                aria-label={t('prefs.fontSmaller')}
              >−</Button>
              <span class="font-mono text-sm min-w-[36px] text-center">{prefs.editor_font_size}px</span>
              <Button
                variant="outline"
                size="icon"
                onclick={() => prefs.editor_font_size = Math.min(20, prefs.editor_font_size + 1)}
                disabled={prefs.editor_font_size >= 20}
                aria-label={t('prefs.fontLarger')}
              >+</Button>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>{t('prefs.afterOpening')}</Label>
          <label class="check-row">
            <Checkbox bind:checked={prefs.close_on_open_editor} />
            <span>{t('prefs.closeAfterEditor')}</span>
          </label>
          <label class="check-row">
            <Checkbox bind:checked={prefs.close_on_open_file} />
            <span>{t('prefs.closeAfterFile')}</span>
          </label>
        </div>
      </TabsContent>

      <!-- Terminal -->
      <TabsContent value="terminal" class="tab-body">
        <div class="field">
          <Label>{t('prefs.preferredTerminal')}</Label>
          {#if terminalEntries.length > 0}
            <div class="radio-group">
              {#each terminalEntries as [name, execPath]}
                <div class="radio-row">
                  <label class="radio-label" title={execPath}>
                    <input type="radio" name="preferred_terminal" value={name} bind:group={prefs.terminal.preferred} />
                    {name}
                  </label>
                  <button class="remove-btn" onclick={() => removeTerminal(name)} title={t('common.remove')}>✕</button>
                </div>
              {/each}
            </div>
          {:else}
            <p class="hint">{t('prefs.noTerminals')}</p>
          {/if}
          {#if prefs.terminal.last_detected}
            <p class="hint">{t('prefs.lastDetected', { date: new Date(prefs.terminal.last_detected).toLocaleString() })}</p>
          {/if}
          <div class="detect-row">
            <Button variant="outline" size="sm" onclick={handleDetectTerminals} disabled={detecting}>
              {#if detecting}<span class="spinner"></span>{/if}
              {detecting ? t('prefs.detecting') : t('prefs.detectTerminals')}
            </Button>
            <Button variant="outline" size="sm" onclick={() => (showAddTerminal = true)}>
              {t('prefs.addTerminal')}
            </Button>
          </div>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>{t('prefs.afterOpening')}</Label>
          <label class="check-row">
            <Checkbox bind:checked={prefs.close_on_open_terminal} />
            <span>{t('prefs.closeAfterTerminal')}</span>
          </label>
        </div>

        {#if detectError}
          <p class="error-msg">{detectError}</p>
        {/if}
      </TabsContent>

      <!-- Claude -->
      <TabsContent value="claude" class="tab-body">
        <div class="field">
          <Label>{t('prefs.claudeProfiles')}</Label>
          <p class="hint">{t('prefs.claudeHint.before')}<code>CLAUDE_CONFIG_DIR</code>{t('prefs.claudeHint.after')}</p>
          {#if profileEntries.length > 0}
            <div class="radio-group">
              {#each profileEntries as [name, dir]}
                <div class="radio-row">
                  <span class="profile-label" title={dir}>
                    <span class="profile-name">{name}</span>
                    <span class="profile-dir">{dir}</span>
                  </span>
                  <button class="remove-btn" onclick={() => removeProfile(name)} title={t('common.remove')}>✕</button>
                </div>
              {/each}
            </div>
          {:else}
            <p class="hint">{t('prefs.noProfiles')}</p>
          {/if}
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label for="claude-profile-name">{t('prefs.addProfile')}</Label>
          <Input id="claude-profile-name" bind:value={newProfileName} placeholder={t('prefs.profileNamePlaceholder')} />
          <div class="input-row">
            <Input bind:value={newProfileDir} placeholder="~/.claude-work" />
            <Button variant="outline" size="sm" onclick={browseProfileDir}>{t('common.browse')}</Button>
          </div>
          <div class="detect-row">
            <Button variant="outline" size="sm" onclick={addProfile}>{t('prefs.addProfileButton')}</Button>
          </div>
          {#if profileError}<p class="error-msg">{profileError}</p>{/if}
        </div>

        {#if detectError}
          <p class="error-msg">{detectError}</p>
        {/if}
      </TabsContent>

      <!-- System -->
      <TabsContent value="system" class="tab-body">
        <div class="field">
          <Label>{t('prefs.startup')}</Label>
          <label class="check-row flex items-start gap-2">
            <Checkbox bind:checked={prefs.autostart} class="mt-1" />
            <div class="flex flex-col">
              <span>{t('prefs.autostart')}</span>
              <span class="text-[0.78rem] text-muted-foreground">{t('prefs.autostartHint')}</span>
            </div>
          </label>
          <div class="sub-field">
            <Label for="close-behavior">{t('prefs.whenClosing')}</Label>
            <select
              id="close-behavior"
              class="native-select"
              value={prefs.show_tray ? 'tray' : 'quit'}
              onchange={(e) => {
                const v = (e.currentTarget as HTMLSelectElement).value;
                prefs.show_tray = v === 'tray';
                prefs.keep_background = v === 'tray';
              }}
            >
              <option value="tray">{t('prefs.closeTray')}</option>
              <option value="quit">{t('prefs.closeQuit')}</option>
            </select>
            <p class="text-[0.78rem] text-muted-foreground">
              {prefs.show_tray ? t('prefs.trayHint') : t('prefs.quitHint')}
            </p>
          </div>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>{t('prefs.hotkey')}</Label>
          <p class="hint">{t('prefs.hotkeyHint')}</p>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div style="display: flex; gap: 8px; align-items: stretch;">
            <div
              class="hotkey-recorder"
              style="flex: 1;"
              class:recording={recordingHotkey}
              tabindex="0"
              role="button"
              aria-label={t('prefs.hotkeyRecordAria')}
              onclick={startRecording}
              onblur={stopRecording}
              onkeydown={handleHotkeyKeydown}
            >
              {#if recordingHotkey}
                <span class="hotkey-recording-label">{t('prefs.hotkeyRecording')}</span>
              {:else}
                <kbd class="hotkey-display">{prefs.hotkey || t('prefs.hotkeyNotSet')}</kbd>
                <span class="hotkey-hint">{t('prefs.hotkeyChange')}</span>
              {/if}
            </div>
            {#if prefs.hotkey && !recordingHotkey}
              <Button variant="outline" onclick={(e) => { e.stopPropagation(); prefs.hotkey = ''; hotkeyError = ''; }}>{t('prefs.disable')}</Button>
            {/if}
          </div>
          {#if hotkeyError}
            <p class="error-msg">{hotkeyError}</p>
          {/if}
          <p class="hint">{t('prefs.hotkeyFormat.before')} <code>Super+Shift+KeyV</code> {t('prefs.hotkeyFormat.after')}</p>
          {#if osType === 'linux'}
            <p class="hint" style="margin-top: 4px; color: #a1a1aa;">
              <strong>{t('prefs.waylandNote.title')}</strong> {t('prefs.waylandNote.before')} <code>vori</code> {t('prefs.waylandNote.after')}
            </p>
          {/if}
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>{t('prefs.updates')}</Label>
          <p class="hint">{t('prefs.currentVersion')} <strong>v{__APP_VERSION__}</strong>{updaterStore.available ? t('prefs.versionAvailable', { version: updaterStore.available.version }) : ''}.</p>
          <div style="display: flex; gap: 8px; align-items: center; margin-top: 4px;">
            <Button variant="outline" onclick={handleCheckForUpdate} disabled={updaterStore.state === 'checking'}>
              {updaterStore.state === 'checking' ? t('prefs.checking') : t('prefs.checkUpdates')}
            </Button>
            {#if updaterStore.state === 'up-to-date'}
              <span class="text-[0.82rem] text-muted-foreground">{t('prefs.upToDate')}</span>
            {:else if updaterStore.state === 'available'}
              <span class="text-[0.82rem]" style="color: var(--color-accent);">{t('prefs.updateAvailable', { version: updaterStore.available?.version ?? '' })}</span>
            {:else if updaterStore.state === 'error'}
              <span class="text-[0.82rem]" style="color: #c0392b;">{t('prefs.checkFailed')}</span>
            {/if}
          </div>
        </div>

        <div class="divider"></div>

        <div class="field">
          <Label>{t('prefs.backup')}</Label>
          <p class="hint">{t('prefs.backupHint')}</p>
          <div style="display: flex; gap: 8px; align-items: center; margin-top: 4px;">
            <Button variant="outline" onclick={handleExport} disabled={backupBusy}>{t('prefs.export')}</Button>
            <Button variant="outline" onclick={handleImport} disabled={backupBusy}>{t('prefs.import')}</Button>
          </div>
          <p class="hint">{t('prefs.configDirHint.before')} <code>VORI_CONFIG_DIR</code> {t('prefs.configDirHint.after')}</p>
        </div>
      </TabsContent>
    </Tabs>

    <DialogFooter class="px-6 pb-5 pt-3 border-t border-border mt-2">
      <Button variant="ghost" onclick={() => dialogStore.close()}>{t('common.cancel')}</Button>
      <Button onclick={handleSave}>{t('common.save')}</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

{#if showAddEditor}
  <AddEditorModal title={t('editorModal.addEditor')} onAdd={handleAddEditor} onClose={() => (showAddEditor = false)} />
{/if}

{#if showAddTerminal}
  <AddEditorModal title={t('editorModal.addTerminal')} onAdd={handleAddTerminal} onClose={() => (showAddTerminal = false)} />
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

  .profile-label { display: flex; flex-direction: column; min-width: 0; flex: 1; font-size: var(--text-base); }
  .profile-name { color: var(--color-text); }
  .profile-dir {
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

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

  .sub-field { display: flex; flex-direction: column; gap: 4px; margin-top: 4px; }

  .pref-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .pref-row :global(label) { flex: 1; margin: 0; }
  .pref-row :global(.native-select),
  .pref-row :global(.flex) { flex: 0 0 auto; }
  .pref-row :global(.native-select) { width: auto; min-width: 110px; padding: 5px 28px 5px 10px; }
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
