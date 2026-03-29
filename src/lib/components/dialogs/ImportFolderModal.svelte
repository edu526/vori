<script lang="ts">
  import { dialogStore } from '$lib/stores/dialogs.svelte';
  import { configStore } from '$lib/stores/config.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import { scanFolder, bulkImportProjects, addCategory } from '$lib/api/commands';
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import StackIcon from '$lib/components/StackIcon.svelte';
  import type { ScannedProject } from '$lib/api/types';

  // ── Tree data model ──────────────────────────────────────────────────────────

  interface FolderNode {
    kind: 'folder';
    name: string;
    /** Relative path from scan root, e.g. "work/backend" */
    folderPath: string;
    children: TreeNode[];
  }

  interface ProjectNode {
    kind: 'project';
    proj: ScannedProject;
  }

  type TreeNode = FolderNode | ProjectNode;

  function buildTree(projects: ScannedProject[]): TreeNode[] {
    const root: FolderNode = { kind: 'folder', name: '', folderPath: '', children: [] };

    for (const proj of projects) {
      const segments = proj.relative_path.split('/');
      let current = root;

      // Navigate/create intermediate folder nodes (all but last segment = project)
      for (let i = 0; i < segments.length - 1; i++) {
        const seg = segments[i];
        let child = current.children.find(
          (c): c is FolderNode => c.kind === 'folder' && c.name === seg,
        );
        if (!child) {
          child = {
            kind: 'folder',
            name: seg,
            folderPath: segments.slice(0, i + 1).join('/'),
            children: [],
          };
          current.children.push(child);
        }
        current = child;
      }

      current.children.push({ kind: 'project', proj });
    }

    return root.children;
  }

  /** Collect all project paths under a tree node */
  function collectPaths(nodes: TreeNode[]): string[] {
    const paths: string[] = [];
    for (const node of nodes) {
      if (node.kind === 'project') paths.push(node.proj.path);
      else paths.push(...collectPaths(node.children));
    }
    return paths;
  }

  // ── Store state ───────────────────────────────────────────────────────────────

  const payload = $derived(
    dialogStore.current?.type === 'import-folder' ? dialogStore.current : null,
  );
  const isOpen = $derived(!!payload);

  let folderPath = $state('');
  let scanning = $state(false);
  let scanned = $state<ScannedProject[]>([]);
  let selected = $state<Set<string>>(new Set());
  let selectedParent = $state('');
  let mode = $state<'flat' | 'nested'>('flat');
  let collapsed = $state<Set<string>>(new Set());
  let importing = $state(false);
  let error = $state('');
  let hasScanned = $state(false);

  const categoryOptions = $derived(
    Object.keys(configStore.categories).sort((a, b) => a.localeCompare(b)),
  );

  const tree = $derived(mode === 'nested' ? buildTree(scanned) : []);

  $effect(() => {
    if (payload) {
      selectedParent = payload.defaultParent ?? '';
      folderPath = '';
      scanned = [];
      selected = new Set();
      collapsed = new Set();
      error = '';
      hasScanned = false;
      scanning = false;
      importing = false;
    }
  });

  // ── Folder checkbox state helpers ─────────────────────────────────────────────

  function folderChecked(node: FolderNode): boolean {
    return collectPaths(node.children).every((p) => selected.has(p));
  }

  function folderIndeterminate(node: FolderNode): boolean {
    const paths = collectPaths(node.children);
    const count = paths.filter((p) => selected.has(p)).length;
    return count > 0 && count < paths.length;
  }

  function toggleFolder(node: FolderNode) {
    const paths = collectPaths(node.children);
    const next = new Set(selected);
    if (paths.every((p) => next.has(p))) {
      paths.forEach((p) => next.delete(p));
    } else {
      paths.forEach((p) => next.add(p));
    }
    selected = next;
  }

  function toggleOne(path: string) {
    const next = new Set(selected);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    selected = next;
  }

  function toggleCollapse(folderPath: string) {
    const next = new Set(collapsed);
    if (next.has(folderPath)) next.delete(folderPath);
    else next.add(folderPath);
    collapsed = next;
  }

  const allSelected = $derived(scanned.length > 0 && selected.size === scanned.length);
  const someSelected = $derived(selected.size > 0 && selected.size < scanned.length);

  function toggleAll() {
    if (selected.size === scanned.length) {
      selected = new Set();
    } else {
      selected = new Set(scanned.map((p) => p.path));
    }
  }

  // ── Scan ──────────────────────────────────────────────────────────────────────

  async function handleBrowse() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const result = await open({ directory: true });
      if (typeof result === 'string') {
        folderPath = result;
        await handleScan();
      }
    } catch (e) {
      console.error('Failed to open directory picker:', e);
    }
  }

  async function handleScan() {
    if (!folderPath) return;
    scanning = true;
    error = '';
    hasScanned = false;
    collapsed = new Set();
    try {
      scanned = await scanFolder(folderPath);
      selected = new Set(scanned.map((p) => p.path));
      hasScanned = true;
      // Auto-set parent to the scanned folder name (existing or will be created on import)
      if (!selectedParent) {
        const folderName = folderPath.split(/[\\/]/).filter(Boolean).pop() ?? '';
        if (folderName) selectedParent = folderName;
      }
    } catch (e) {
      error = String(e);
    } finally {
      scanning = false;
    }
  }

  // ── Import ────────────────────────────────────────────────────────────────────

  /** Compute full-path key for an intermediate folder segment.
   *  e.g. selectedParent="PERSONALES", segments=["work","backend"], i=1
   *  → "PERSONALES/work/backend"
   */
  function segmentKey(segments: string[], i: number): string {
    const parts = selectedParent
      ? [selectedParent, ...segments.slice(0, i + 1)]
      : segments.slice(0, i + 1);
    return parts.join('/');
  }

  function buildCategoryPlan(): { key: string; parent: string | null }[] {
    const plan = new Map<string, string | null>();
    for (const proj of scanned.filter((p) => selected.has(p.path))) {
      const segments = proj.relative_path.split('/');
      for (let i = 0; i < segments.length - 1; i++) {
        // Full-path key avoids same-name collisions across branches (Bug 2 & 3)
        const key = segmentKey(segments, i);
        if (!plan.has(key) && !configStore.categories[key]) {
          const parent = i === 0 ? (selectedParent || null) : segmentKey(segments, i - 1);
          plan.set(key, parent);
        }
      }
    }
    return Array.from(plan.entries()).map(([key, parent]) => ({ key, parent }));
  }

  /** Returns a unique project key, preferring p.name, then parentFolder-name, then full slug. */
  function uniqueProjectKey(p: ScannedProject, usedKeys: Set<string>): string {
    if (!usedKeys.has(p.name)) return p.name;
    const segments = p.relative_path.split('/');
    if (segments.length > 1) {
      const candidate = `${segments[segments.length - 2]}-${p.name}`;
      if (!usedKeys.has(candidate)) return candidate;
    }
    return p.relative_path.replace(/\//g, '-');
  }

  async function handleImport() {
    if (mode === 'flat' && !selectedParent) { error = 'Select a target category first.'; return; }
    if (selected.size === 0) { error = 'Select at least one project.'; return; }

    importing = true;
    error = '';
    try {
      // Create the root category if it doesn't exist yet
      if (selectedParent && !configStore.categories[selectedParent]) {
        await addCategory(selectedParent, null);
      }

      if (mode === 'nested') {
        for (const { key, parent } of buildCategoryPlan()) {
          await addCategory(key, parent);
        }
        const usedKeys = new Set(Object.keys(configStore.projects));
        const entries: [string, { path: string; parent: string; stack?: string }][] =
          scanned.filter((p) => selected.has(p.path)).map((p) => {
            const segments = p.relative_path.split('/');
            const directParent =
              segments.length > 1
                ? segmentKey(segments, segments.length - 2)
                : (selectedParent || '');
            const key = uniqueProjectKey(p, usedKeys);
            usedKeys.add(key);
            return [key, { path: p.path, parent: directParent, stack: p.stack !== 'unknown' ? p.stack : undefined }];
          });
        await bulkImportProjects(entries);
      } else {
        const usedKeys = new Set(Object.keys(configStore.projects));
        const entries: [string, { path: string; parent: string; stack?: string }][] =
          scanned.filter((p) => selected.has(p.path)).map((p) => {
            const key = uniqueProjectKey(p, usedKeys);
            usedKeys.add(key);
            return [key, { path: p.path, parent: selectedParent, stack: p.stack !== 'unknown' ? p.stack : undefined }];
          });
        await bulkImportProjects(entries);
      }

      await configStore.load();
      navigationStore.refresh(
        configStore.categories, configStore.projects,
        configStore.files, configStore.favorites, configStore.recents,
      );
      dialogStore.close();
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }
</script>

<!-- ── Recursive tree snippet ─────────────────────────────────────────────── -->
{#snippet treeNodes(nodes: TreeNode[], depth: number)}
  {#each nodes as node (node.kind === 'project' ? node.proj.path : node.folderPath)}
    {#if node.kind === 'folder'}
      {@const isCollapsed = collapsed.has(node.folderPath)}
      <li class="tree-row folder-row" style="--depth: {depth}">
        <button class="collapse-btn" onclick={() => toggleCollapse(node.folderPath)} aria-label="toggle">
          <svg class="chevron" class:open={!isCollapsed} width="10" height="10" viewBox="0 0 10 10">
            <path d="M3 2l4 3-4 3" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <input
          type="checkbox"
          checked={folderChecked(node)}
          indeterminate={folderIndeterminate(node)}
          onchange={() => toggleFolder(node)}
        />
        <svg class="folder-icon" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1.5 3A1.5 1.5 0 0 1 3 1.5h3.879a1.5 1.5 0 0 1 1.06.44l1.122 1.12H13A1.5 1.5 0 0 1 14.5 4.5v8A1.5 1.5 0 0 1 13 14H3A1.5 1.5 0 0 1 1.5 12.5V3z"/>
        </svg>
        <span class="node-name folder-name">{node.name}</span>
        <span class="node-count">{collectPaths(node.children).length}</span>
      </li>
      {#if !isCollapsed}
        <ul class="tree-children">
          {@render treeNodes(node.children, depth + 1)}
        </ul>
      {/if}
    {:else}
      <li class="tree-row project-row" style="--depth: {depth}">
        <span class="collapse-btn"></span><!-- spacer -->
        <input
          type="checkbox"
          checked={selected.has(node.proj.path)}
          onchange={() => toggleOne(node.proj.path)}
        />
        <svg class="project-icon" width="14" height="14" viewBox="0 0 16 16">
          <path fill="currentColor" d="M1.5 3A1.5 1.5 0 0 1 3 1.5h3.879a1.5 1.5 0 0 1 1.06.44l1.122 1.12H13A1.5 1.5 0 0 1 14.5 4.5v8A1.5 1.5 0 0 1 13 14H3A1.5 1.5 0 0 1 1.5 12.5V3z"/>
          <path fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" style="opacity:0.5" d="M5.8 10 4.5 8.7 5.8 7.4M10.2 10l1.3-1.3-1.3-1.3"/>
        </svg>
        <span class="node-name">{node.proj.name}</span>
        <StackIcon stack={node.proj.stack} size={12} />
      </li>
    {/if}
  {/each}
{/snippet}

<!-- ── Dialog ─────────────────────────────────────────────────────────────── -->
<Dialog open={isOpen} onOpenChange={(o) => { if (!o) dialogStore.close(); }}>
  <DialogContent class="w-[560px] max-w-[95vw]" showCloseButton={false} style="max-height: min(90vh, 680px); display: flex; flex-direction: column; overflow: hidden;">
    <DialogHeader style="flex-shrink: 0;">
      <DialogTitle>Import projects from folder</DialogTitle>
    </DialogHeader>

    <div class="body">
      <!-- Fixed controls (never scroll away) -->
      <div class="controls">
        <!-- Folder picker -->
        <div class="row">
          <div class="path-display" class:empty={!folderPath}>
            {folderPath || 'No folder selected'}
          </div>
          <Button variant="outline" size="sm" onclick={handleBrowse} disabled={scanning}>Browse</Button>
        </div>

        <!-- Mode toggle -->
        <div class="mode-toggle">
          <button class="mode-btn" class:active={mode === 'flat'} onclick={() => { mode = 'flat'; }}>Flat</button>
          <button class="mode-btn" class:active={mode === 'nested'} onclick={() => { mode = 'nested'; }}>Nested</button>
        </div>

        <!-- Category selector -->
        <div class="field">
          <label class="field-label" for="import-parent">
            {mode === 'flat' ? 'Import into category' : 'Root category (folder structure goes inside)'}
          </label>
          <input
            id="import-parent"
            list="import-parent-list"
            bind:value={selectedParent}
            placeholder="category name (existing or new)"
            class="native-input"
          />
          <datalist id="import-parent-list">
            {#each categoryOptions as catKey}
              <option value={catKey}>{catKey}</option>
            {/each}
          </datalist>
          {#if mode === 'nested'}
            <span class="nested-hint">Intermediate folders will be created as sub-categories inside this category</span>
          {/if}
        </div>
      </div>

      <!-- Scrollable results -->
      <div class="results-scroll">
        {#if scanning}
          <div class="state-row"><span class="spinner"></span> Scanning…</div>
        {:else if hasScanned && scanned.length === 0}
          <div class="state-row">No projects detected in that folder.</div>
        {:else if scanned.length > 0}
          <div class="results-header">
            <label class="check-all">
              <input type="checkbox" checked={allSelected} indeterminate={someSelected} onchange={toggleAll} />
              <span>{selected.size} of {scanned.length} selected</span>
            </label>
          </div>

          {#if mode === 'nested'}
            <!-- Tree view -->
            <ul class="tree">
              {@render treeNodes(tree, 0)}
            </ul>
          {:else}
            <!-- Flat list -->
            <ul class="flat-list">
              {#each scanned as proj (proj.path)}
                <li class="flat-item">
                  <label class="flat-label">
                    <input type="checkbox" checked={selected.has(proj.path)} onchange={() => toggleOne(proj.path)} />
                    <span class="node-name">{proj.name}</span>
                    <StackIcon stack={proj.stack} size={13} />
                    <span class="flat-path">{proj.path.replace(/^\/home\/[^/]+/, '~')}</span>
                  </label>
                </li>
              {/each}
            </ul>
          {/if}
        {/if}

        {#if error}<p class="error-msg">{error}</p>{/if}
      </div>
    </div>

    <DialogFooter style="flex-shrink: 0;">
      <Button variant="ghost" onclick={() => dialogStore.close()}>Cancel</Button>
      <Button
        onclick={handleImport}
        disabled={selected.size === 0 || (mode === 'flat' && !selectedParent) || importing}
      >
        {importing ? 'Importing…' : `Import ${selected.size > 0 ? selected.size : ''} project${selected.size !== 1 ? 's' : ''}`}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex-shrink: 0;
  }

  .results-scroll {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .path-display {
    flex: 1;
    font-size: var(--text-sm);
    padding: 6px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .path-display.empty { color: var(--color-text-secondary); font-style: italic; }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .mode-toggle {
    display: flex;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    overflow: hidden;
    width: fit-content;
    background: var(--color-hover);
  }

  .mode-btn {
    padding: 4px 12px;
    font-size: var(--text-sm);
    font-family: inherit;
    border: none;
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
    white-space: nowrap;
  }
  .mode-btn:not(:last-child) { border-right: 1px solid var(--color-border); }
  .mode-btn.active { background: var(--color-accent); color: var(--color-accent-fg); }


  .native-input {
    width: 100%;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: var(--text-base);
    font-family: inherit;
    padding: 5px 10px;
    outline: none;
    transition: border-color 0.15s;
  }
  .native-input:focus { border-color: var(--color-accent); }
  .native-input::placeholder { color: var(--color-text-secondary); }

  .nested-hint {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    font-style: italic;
  }

  .state-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--text-base);
    color: var(--color-text-secondary);
    padding: 8px 0;
  }

  .results-header {
    display: flex;
    align-items: center;
    padding: 4px 0 2px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .check-all {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
    user-select: none;
  }

  /* ── Tree ── */
  .tree,
  .tree-children {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .tree-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px 4px calc(8px + var(--depth, 0) * 18px);
    border-radius: 5px;
    transition: background 0.08s;
    cursor: default;
  }
  .tree-row:hover { background: var(--color-hover); }

  .collapse-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: var(--color-text-secondary);
  }

  .chevron {
    transition: transform 0.15s;
    transform: rotate(0deg);
  }
  .chevron.open { transform: rotate(90deg); }

  .folder-icon {
    flex-shrink: 0;
    color: var(--color-accent);
    opacity: 0.8;
  }

  .project-icon {
    flex-shrink: 0;
    color: var(--color-text-secondary);
    opacity: 0.6;
  }

  .node-name {
    font-size: var(--text-base);
    color: var(--color-text);
    flex-shrink: 0;
  }

  .folder-name { font-weight: 600; }

  .node-count {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    background: var(--color-hover);
    border-radius: 10px;
    padding: 1px 6px;
    flex-shrink: 0;
  }

  /* ── Flat list ── */
  .flat-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .flat-item { border-radius: 5px; transition: background 0.08s; }
  .flat-item:hover { background: var(--color-hover); }

  .flat-label {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    cursor: pointer;
    width: 100%;
  }

  .flat-path {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    text-align: right;
  }

  .error-msg {
    font-size: var(--text-sm);
    color: #e53e3e;
    margin: 0;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
