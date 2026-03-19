<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let {
    onopenpreferences,
    onsearchopen,
  }: {
    onopenpreferences: () => void;
    onsearchopen: () => void;
  } = $props();

  const win = getCurrentWindow();
  let isMaximized = $state(false);

  $effect(() => {
    win.isMaximized().then((v) => (isMaximized = v));
  });

  async function toggleMaximize() {
    await win.toggleMaximize();
    isMaximized = await win.isMaximized();
  }

  function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as Element).closest('button, input, a, kbd')) return;

    const startX = e.clientX;
    const startY = e.clientY;

    function onMove(me: MouseEvent) {
      if (Math.abs(me.clientX - startX) > 3 || Math.abs(me.clientY - startY) > 3) {
        cleanup();
        win.startDragging?.();
      }
    }

    function cleanup() {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', cleanup);
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', cleanup);
  }

  function handleDblClick(e: MouseEvent) {
    if ((e.target as Element).closest('button, input, a, kbd')) return;
    toggleMaximize();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="toolbar" onmousedown={handleMouseDown} ondblclick={handleDblClick}>
  <!-- Left: app name -->
  <div class="toolbar-left">
    <span class="app-name">Vori</span>
  </div>

  <!-- Center: search trigger -->
  <div class="toolbar-center">
    <button class="search-trigger" onclick={onsearchopen}>
      <svg class="search-icon" width="13" height="13" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/>
        <line x1="10.5" y1="10.5" x2="14" y2="14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <span class="search-placeholder">Search projects, files, categories...</span>
      <kbd>⌘F</kbd>
    </button>
  </div>

  <!-- Right: preferences + window controls -->
  <div class="toolbar-right">
    <button class="toolbar-btn" onclick={onopenpreferences} title="Preferences">
      <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <circle cx="8" cy="8" r="2.25" stroke="currentColor" stroke-width="1.25"/>
        <path d="M8 1.5v1M8 13.5v1M1.5 8h1M13.5 8h1M3.404 3.404l.707.707M11.889 11.889l.707.707M3.404 12.596l.707-.707M11.889 4.111l.707-.707" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
      </svg>
      <span>Preferences</span>
    </button>

    <div class="divider"></div>

    <button class="wc-btn" onclick={() => win.minimize()} title="Minimize" aria-label="Minimize">
      <svg width="11" height="2" viewBox="0 0 11 2" fill="currentColor" aria-hidden="true">
        <rect width="11" height="2" rx="1"/>
      </svg>
    </button>
    <button class="wc-btn" onclick={toggleMaximize} title={isMaximized ? 'Restore' : 'Maximize'} aria-label={isMaximized ? 'Restore' : 'Maximize'}>
      {#if isMaximized}
        <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="3" y="0" width="8" height="8" rx="1.2"/>
          <path d="M1 3v5.5A1.5 1.5 0 0 0 2.5 10H8"/>
        </svg>
      {:else}
        <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="0.65" y="0.65" width="9.7" height="9.7" rx="1.5"/>
        </svg>
      {/if}
    </button>
    <button class="wc-btn close" onclick={() => win.close()} title="Close" aria-label="Close">
      <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
        <line x1="1.5" y1="1.5" x2="9.5" y2="9.5"/>
        <line x1="9.5" y1="1.5" x2="1.5" y2="9.5"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    height: 40px;
    padding: 0 6px 0 14px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
  }

  .app-name {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    opacity: 0.5;
  }

  .toolbar-center {
    display: flex;
    justify-content: center;
    min-width: 0;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    justify-content: flex-end;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.1s, color 0.1s;
  }

  .toolbar-btn:hover {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .divider {
    width: 1px;
    height: 16px;
    background: var(--color-border);
    margin: 0 4px;
    flex-shrink: 0;
  }

  /* Window controls */
  .wc-btn {
    width: 28px;
    height: 26px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.45;
    transition: opacity 0.1s, background 0.1s, color 0.1s;
  }

  .wc-btn:hover {
    opacity: 1;
    background: var(--color-hover);
    color: var(--color-text);
  }

  .wc-btn.close:hover {
    background: #e05252;
    color: #fff;
    opacity: 1;
  }

  /* Search trigger */
  .search-trigger {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px;
    height: 26px;
    width: 100%;
    max-width: 360px;
    background: var(--color-hover);
    border: 1px solid var(--color-border);
    border-radius: 5px;
    cursor: pointer;
    color: var(--color-text-secondary);
    font-size: 12px;
    font-family: inherit;
    text-align: left;
    transition: border-color 0.1s;
    user-select: none;
  }

  .search-trigger:hover {
    border-color: var(--color-accent);
  }

  .search-icon {
    flex-shrink: 0;
  }

  .search-placeholder {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .search-trigger kbd {
    font-size: 10px;
    padding: 1px 5px;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    background: var(--color-surface);
    color: var(--color-text-secondary);
    flex-shrink: 0;
    font-family: inherit;
  }
</style>
