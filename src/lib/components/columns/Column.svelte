<script lang="ts">
  import type { Column, NavItem } from '$lib/stores/navigation.svelte';
  import ColumnItem from './ColumnItem.svelte';

  let {
    column,
    columnIndex,
    onselect,
    onrightclick,
    onemptyrightclick,
    onopen,
  }: {
    column: Column;
    columnIndex: number;
    onselect: (columnIndex: number, key: string) => void;
    onrightclick: (item: NavItem, x: number, y: number) => void;
    onemptyrightclick: (columnIndex: number, x: number, y: number) => void;
    onopen: (item: NavItem) => void;
  } = $props();

  function handleContextMenu(e: MouseEvent) {
    if ((e.target as Element).closest('.column-item')) return; // ColumnItem handles it
    e.preventDefault();
    onemptyrightclick(columnIndex, e.clientX, e.clientY);
  }
</script>

<div class="column" role="region" oncontextmenu={handleContextMenu}>
  {#if column.title}
    <div class="column-header">{column.title}</div>
  {/if}
  <div class="column-scroll">
    {#each column.items as item (item.key)}
      <ColumnItem
        {item}
        selected={column.selectedKey === item.key}
        onselect={() => onselect(columnIndex, item.key)}
        onrightclick={onrightclick}
        {onopen}
      />
    {/each}
    {#if column.items.length === 0}
      <div class="empty-state">No items</div>
    {/if}
  </div>
</div>

<style>
  .column {
    display: flex;
    flex-direction: column;
    width: 220px;
    flex-shrink: 0;
    border-right: 1px solid var(--color-border);
    overflow: hidden;
  }

  .column-header {
    padding: 6px 10px;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    border-bottom: 1px solid var(--color-border);
    user-select: none;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .column-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .empty-state {
    padding: 16px 10px;
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    text-align: center;
    font-style: italic;
  }
</style>
