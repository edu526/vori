<script lang="ts">
  import { search } from '$lib/api/commands';
  import type { SearchResult } from '$lib/api/types';
  import ItemIcon from '$lib/components/ItemIcon.svelte';

  let {
    onclose,
    onresult,
  }: {
    onclose: () => void;
    onresult: (result: SearchResult) => void;
  } = $props();

  let inputEl = $state<HTMLInputElement | null>(null);
  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let highlightedIndex = $state(0);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    inputEl?.focus();
  });

  function handleInput(e: Event) {
    query = (e.target as HTMLInputElement).value;
    highlightedIndex = 0;
    if (debounceTimer) clearTimeout(debounceTimer);
    if (!query.trim()) { results = []; return; }
    debounceTimer = setTimeout(async () => {
      try {
        results = await search(query.trim());
        highlightedIndex = 0;
      } catch {
        results = [];
      }
    }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { onclose(); return; }
    if (e.key === 'ArrowDown') { e.preventDefault(); highlightedIndex = Math.min(highlightedIndex + 1, results.length - 1); scrollToHighlighted(); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); highlightedIndex = Math.max(highlightedIndex - 1, 0); scrollToHighlighted(); }
    else if (e.key === 'Enter' && results[highlightedIndex]) { select(results[highlightedIndex]); }
  }

  function scrollToHighlighted() {
    setTimeout(() => {
      document.querySelector(`[data-search-index="${highlightedIndex}"]`)?.scrollIntoView({ block: 'nearest' });
    }, 0);
  }

  function select(result: SearchResult) {
    onresult(result);
    onclose();
  }

  const TYPE_LABELS: Record<string, string> = {
    category: 'Category', subcategory: 'Subcategory', project: 'Project', file: 'File',
  };
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="overlay"
  role="dialog"
  aria-modal="true"
  onmousedown={(e) => { if (e.target === e.currentTarget) onclose(); }}
>
  <div class="modal">
    <div class="input-row">
      <svg class="search-icon" width="18" height="18" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/>
        <line x1="10.5" y1="10.5" x2="14" y2="14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <input
        bind:this={inputEl}
        type="text"
        class="search-input"
        placeholder="Search projects, files, categories..."
        value={query}
        oninput={handleInput}
        onkeydown={handleKeydown}
      />
      <kbd>Esc</kbd>
    </div>

    {#if results.length > 0}
      <div class="divider"></div>
      <ul class="results">
        {#each results as result, i (result.key)}
          <li>
            <button
              class="result-item"
              class:highlighted={i === highlightedIndex}
              data-search-index={i}
              onmouseenter={() => { highlightedIndex = i; }}
              onclick={() => select(result)}
            >
              <span class="result-icon">
                <ItemIcon type={result.result_type as 'category' | 'subcategory' | 'project' | 'file'} size={15} />
              </span>
              <span class="result-name">{result.name}</span>
              {#if result.path}
                <span class="result-path">{result.path.replace(/^\/home\/[^/]+/, '~')}</span>
              {/if}
              <span class="result-type">{TYPE_LABELS[result.result_type] ?? result.result_type}</span>
            </button>
          </li>
        {/each}
      </ul>
    {:else if query.trim()}
      <div class="divider"></div>
      <p class="empty">No results for <strong>{query}</strong></p>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.35);
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 12vh;
  }

  .modal {
    width: 580px;
    max-width: calc(100vw - 48px);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
  }

  .search-icon {
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 16px;
    color: var(--color-text);
    outline: none;
    min-width: 0;
  }

  .search-input::placeholder {
    color: var(--color-text-secondary);
  }

  kbd {
    font-size: 11px;
    padding: 2px 6px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text-secondary);
    background: var(--color-hover);
    flex-shrink: 0;
    font-family: inherit;
  }

  .divider {
    height: 1px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  .results {
    list-style: none;
    margin: 0;
    padding: 6px;
    overflow-y: auto;
    flex: 1;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    color: var(--color-text);
  }

  .result-item.highlighted {
    background: var(--color-accent);
    color: white;
  }

  .result-icon {
    flex-shrink: 0;
    display: flex;
    opacity: 0.7;
  }

  .result-item.highlighted .result-icon {
    opacity: 1;
  }

  .result-name {
    font-size: 14px;
    font-weight: 500;
    flex-shrink: 0;
  }

  .result-path {
    font-size: 12px;
    color: var(--color-text-secondary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .result-item.highlighted .result-path {
    color: rgba(255, 255, 255, 0.7);
  }

  .result-type {
    font-size: 11px;
    padding: 2px 7px;
    border-radius: 4px;
    background: var(--color-hover);
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .result-item.highlighted .result-type {
    background: rgba(255, 255, 255, 0.2);
    color: rgba(255, 255, 255, 0.85);
  }

  .empty {
    padding: 20px 16px;
    font-size: 13px;
    color: var(--color-text-secondary);
    text-align: center;
    margin: 0;
  }
</style>
