<script lang="ts">
  import type { NavItem } from '$lib/stores/navigation.svelte';
  import ItemIcon from '$lib/components/ItemIcon.svelte';

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
      <ItemIcon type={item.type} size={14} />
    </span>
    <span class="label">{item.label}</span>
    {#if item.isFavorite}
      <span class="favorite">★</span>
    {/if}
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

  .favorite {
    flex-shrink: 0;
    font-size: 0.7rem;
    opacity: 0.6;
    color: var(--color-accent);
  }

  .column-item.selected .favorite {
    color: white;
    opacity: 0.9;
  }
</style>
