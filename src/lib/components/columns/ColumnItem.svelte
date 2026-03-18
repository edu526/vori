<script lang="ts">
  import type { NavItem } from '$lib/stores/navigation.svelte';

  let {
    item,
    selected,
    onselect,
    onrightclick,
  }: {
    item: NavItem;
    selected: boolean;
    onselect: () => void;
    onrightclick: (item: NavItem, x: number, y: number) => void;
  } = $props();
</script>

{#if item.type === 'section-header'}
  <div class="section-header">{item.label}</div>
{:else}
  <button
    class="column-item"
    class:selected
    onclick={onselect}
    oncontextmenu={(e) => { e.preventDefault(); onrightclick(item, e.clientX, e.clientY); }}
    title={item.path}
  >
    <span class="icon">
      {#if item.type === 'category' || item.type === 'subcategory'}
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M1 3a1 1 0 0 1 1-1h3.586a1 1 0 0 1 .707.293L7 3h5a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V3z"/>
        </svg>
      {:else if item.type === 'project'}
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M2 2a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V2zm2 1v8h6V3H4z"/>
        </svg>
      {:else if item.type === 'file'}
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M3 1a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V5.414A1 1 0 0 0 11.707 4.707L8.293 1.293A1 1 0 0 0 7.586 1H3zm5 1.414L10.586 5H8.5a.5.5 0 0 1-.5-.5V2.414z"/>
        </svg>
      {/if}
    </span>
    <span class="label">{item.label}</span>
    {#if item.hasChildren}
      <span class="chevron">›</span>
    {/if}
  </button>
{/if}

<style>
  .section-header {
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 10px 10px 3px;
    color: var(--color-text-secondary);
    user-select: none;
  }

  .column-item {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 100%;
    padding: 5px 10px;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    border-radius: 5px;
    font-size: 0.875rem;
    color: var(--color-text);
    transition: background 0.08s;
  }

  .column-item:hover {
    background: var(--color-hover);
  }

  .column-item.selected {
    background: var(--color-accent);
    color: white;
  }

  .icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    opacity: 0.7;
  }

  .column-item.selected .icon {
    opacity: 1;
  }

  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chevron {
    flex-shrink: 0;
    opacity: 0.4;
    font-size: 1.1rem;
    line-height: 1;
  }

  .column-item.selected .chevron {
    opacity: 0.8;
  }
</style>
