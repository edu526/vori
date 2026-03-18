<script lang="ts">
  import { search } from '$lib/api/commands';
  import type { SearchResult } from '$lib/api/types';

  let {
    onnewcategory,
    onnewproject,
    onnewfile,
    onopenpreferences,
    onsearchresult,
  }: {
    onnewcategory: () => void;
    onnewproject: () => void;
    onnewfile: () => void;
    onopenpreferences: () => void;
    onsearchresult: (result: SearchResult) => void;
  } = $props();

  let searchInputEl = $state<HTMLInputElement | null>(null);

  export function focusSearch() {
    searchInputEl?.focus();
    searchInputEl?.select();
  }

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let dropdownVisible = $state(false);
  let highlightedIndex = $state(-1);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let dropdownEl = $state<HTMLDivElement | null>(null);

  function handleInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    query = value;
    highlightedIndex = -1;

    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
    }

    if (value.trim() === '') {
      results = [];
      dropdownVisible = false;
      return;
    }

    debounceTimer = setTimeout(async () => {
      try {
        const res = await search(value.trim());
        results = res;
        dropdownVisible = res.length > 0;
        highlightedIndex = -1;
      } catch {
        results = [];
        dropdownVisible = false;
      }
    }, 200);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!dropdownVisible) {
      if (e.key === 'Escape') {
        query = '';
        results = [];
      }
      return;
    }

    if (e.key === 'Escape') {
      dropdownVisible = false;
      highlightedIndex = -1;
      e.preventDefault();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightedIndex = Math.min(highlightedIndex + 1, results.length - 1);
      scrollHighlightedIntoView();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightedIndex = Math.max(highlightedIndex - 1, 0);
      scrollHighlightedIntoView();
    } else if (e.key === 'Enter' && highlightedIndex >= 0) {
      e.preventDefault();
      selectResult(results[highlightedIndex]);
    }
  }

  function scrollHighlightedIntoView() {
    // Run after DOM update
    setTimeout(() => {
      const el = dropdownEl?.querySelector(`[data-index="${highlightedIndex}"]`);
      el?.scrollIntoView({ block: 'nearest' });
    }, 0);
  }

  function selectResult(result: SearchResult) {
    dropdownVisible = false;
    highlightedIndex = -1;
    query = '';
    results = [];
    onsearchresult(result);
  }

  function handleResultClick(result: SearchResult) {
    selectResult(result);
  }

  function handleBlur() {
    // Delay hiding so click on result can register first
    setTimeout(() => {
      dropdownVisible = false;
      highlightedIndex = -1;
    }, 150);
  }

  function handleFocus() {
    if (query.trim() !== '' && results.length > 0) {
      dropdownVisible = true;
    }
  }

</script>

<div class="toolbar">
  <!-- Left: action buttons -->
  <div class="toolbar-left">
    <button class="toolbar-btn" onclick={onnewcategory} title="New Category">
      <!-- Folder+ icon -->
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
        <path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h3.086a1.5 1.5 0 0 1 1.06.44L7.5 3.293A1.5 1.5 0 0 0 8.56 3.733H13.5A1.5 1.5 0 0 1 15 5.232V12.5A1.5 1.5 0 0 1 13.5 14h-11A1.5 1.5 0 0 1 1 12.5V3.5Z" stroke="currentColor" stroke-width="1.25" fill="none"/>
        <line x1="8" y1="7.5" x2="8" y2="11.5" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
        <line x1="6" y1="9.5" x2="10" y2="9.5" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
      </svg>
      <span>New Category</span>
    </button>

    <button class="toolbar-btn" onclick={onnewproject} title="New Project">
      <!-- File+ icon -->
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
        <path d="M9 1H3.5A1.5 1.5 0 0 0 2 2.5v11A1.5 1.5 0 0 0 3.5 15h9A1.5 1.5 0 0 0 14 13.5V6L9 1Z" stroke="currentColor" stroke-width="1.25" fill="none"/>
        <polyline points="9,1 9,6 14,6" stroke="currentColor" stroke-width="1.25" fill="none" stroke-linejoin="round"/>
        <line x1="8" y1="9" x2="8" y2="13" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
        <line x1="6" y1="11" x2="10" y2="11" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
      </svg>
      <span>New Project</span>
    </button>

    <button class="toolbar-btn" onclick={onnewfile} title="New File">
      <!-- Doc icon -->
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
        <path d="M9 1H3.5A1.5 1.5 0 0 0 2 2.5v11A1.5 1.5 0 0 0 3.5 15h9A1.5 1.5 0 0 0 14 13.5V6L9 1Z" stroke="currentColor" stroke-width="1.25" fill="none"/>
        <polyline points="9,1 9,6 14,6" stroke="currentColor" stroke-width="1.25" fill="none" stroke-linejoin="round"/>
        <line x1="5" y1="9.5" x2="11" y2="9.5" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
        <line x1="5" y1="11.5" x2="9" y2="11.5" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
      </svg>
      <span>New File</span>
    </button>
  </div>

  <!-- Center: search -->
  <div class="toolbar-center">
    <div class="search-wrapper">
      <div class="search-input-row">
        <svg class="search-icon" width="14" height="14" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
          <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/>
          <line x1="10.5" y1="10.5" x2="14" y2="14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <input
          bind:this={searchInputEl}
          type="text"
          class="search-input"
          placeholder="Search projects..."
          value={query}
          oninput={handleInput}
          onkeydown={handleKeydown}
          onblur={handleBlur}
          onfocus={handleFocus}
        />
      </div>

      {#if dropdownVisible}
        <div class="search-dropdown" bind:this={dropdownEl}>
          {#each results as result, i (result.key)}
            {#if i === 0 || results[i - 1].result_type !== result.result_type}
              <div class="dropdown-section-header">
                {result.result_type === 'category' ? 'Categories' : result.result_type === 'project' ? 'Projects' : 'Files'}
              </div>
            {/if}
            <button
              class="dropdown-item"
              class:highlighted={i === highlightedIndex}
              data-index={i}
              onmousedown={() => handleResultClick(result)}
              onmouseenter={() => { highlightedIndex = i; }}
            >
              <span class="item-name">{result.name}</span>
              {#if result.path}
                <span class="item-path">{result.path}</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Right: preferences -->
  <div class="toolbar-right">
    <button class="toolbar-btn" onclick={onopenpreferences} title="Preferences">
      <!-- Gear icon -->
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
        <circle cx="8" cy="8" r="2.25" stroke="currentColor" stroke-width="1.25"/>
        <path d="M8 1.5v1M8 13.5v1M1.5 8h1M13.5 8h1M3.404 3.404l.707.707M11.889 11.889l.707.707M3.404 12.596l.707-.707M11.889 4.111l.707-.707" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
      </svg>
      <span>Preferences</span>
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    height: 40px;
    padding: 0 8px;
    gap: 8px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .toolbar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    min-width: 0;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.1s;
  }

  .toolbar-btn:hover {
    background: var(--color-hover);
  }

  .toolbar-btn svg {
    flex-shrink: 0;
  }

  /* Search */

  .search-wrapper {
    position: relative;
    width: 100%;
    max-width: 360px;
  }

  .search-input-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    height: 26px;
    background: var(--color-hover);
    border: 1px solid var(--color-border);
    border-radius: 5px;
  }

  .search-icon {
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 12px;
    outline: none;
    min-width: 0;
  }

  .search-input::placeholder {
    color: var(--color-text-secondary);
  }

  /* Dropdown */

  .search-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 9999;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
    max-height: 320px;
    overflow-y: auto;
    padding: 4px 0;
  }

  .dropdown-section-header {
    padding: 4px 10px 2px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    user-select: none;
  }

  .dropdown-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 5px 10px;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    gap: 1px;
    transition: background 0.1s;
  }

  .dropdown-item:hover,
  .dropdown-item.highlighted {
    background: var(--color-hover);
  }

  .item-name {
    font-weight: 500;
    color: var(--color-text);
  }

  .item-path {
    font-size: 10px;
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
</style>
