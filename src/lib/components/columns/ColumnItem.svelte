<script lang="ts">
  import type { NavItem } from '$lib/stores/navigation.svelte';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import ItemIcon from '$lib/components/ItemIcon.svelte';
  import StackIcon from '$lib/components/StackIcon.svelte';
  import { gitStore } from '$lib/stores/git.svelte';

  let {
    item,
    selected,
    active,
    onselect,
    onrightclick,
    onopen,
  }: {
    item: NavItem;
    selected: boolean;
    active: boolean;
    onselect: () => void;
    onrightclick: (item: NavItem, x: number, y: number) => void;
    onopen: (item: NavItem) => void;
  } = $props();

  let el = $state<HTMLElement | null>(null);

  let workspaceSelected = $derived(
    (item.type === 'project' || item.type === 'workspace') &&
    !!item.path &&
    navigationStore.workspaceSelection.has(item.path),
  );

  const git = $derived(item.type === 'project' && item.path ? gitStore.info[item.path] : undefined);
  const tooltip = $derived(
    git
      ? [
          item.path,
          `Branch: ${git.branch}`,
          git.dirty ? 'Uncommitted changes' : 'Working tree clean',
          git.ahead ? `${git.ahead} ahead` : '',
          git.behind ? `${git.behind} behind` : '',
        ]
          .filter(Boolean)
          .join('\n')
      : item.path,
  );

  $effect(() => {
    if (selected && el) {
      el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });

  function handleClick(e: MouseEvent) {
    if ((e.ctrlKey || e.metaKey) && (item.type === 'project' || item.type === 'workspace') && item.path) {
      e.stopPropagation();
      navigationStore.toggleWorkspaceItem(item.key, item.path, item.label);
      return;
    }
    onselect();
    // Single click on a file opens it (in-app for text, externally for binary).
    if (item.type === 'file') {
      onopen(item);
    }
  }
</script>

<div
  bind:this={el}
  class="column-item"
  class:selected
  class:inactive={selected && !active}
  class:workspace-selected={workspaceSelected}
  role="button"
  tabindex="0"
  onclick={handleClick}
  ondblclick={() => { if (item.type === 'project' || item.type === 'workspace') onopen(item); }}
  oncontextmenu={(e) => { e.preventDefault(); onrightclick(item, e.clientX, e.clientY); }}
  onkeydown={(e) => { if (e.key === 'Enter' && (item.type === 'project' || item.type === 'file' || item.type === 'workspace')) { e.preventDefault(); onopen(item); } }}
  title={tooltip}
>
  <span class="icon">
    <ItemIcon type={item.type} size={14} />
  </span>
  <span class="label">{item.label}</span>
  {#if git}
    <span class="git">
      <span class="branch">{git.branch}</span>
      {#if git.dirty}<span class="dirty" aria-label="Uncommitted changes">●</span>{/if}
      {#if git.ahead}<span class="sync">↑{git.ahead}</span>{/if}
      {#if git.behind}<span class="sync">↓{git.behind}</span>{/if}
    </span>
  {/if}
  {#if item.stack}
    <StackIcon stack={item.stack} size={13} />
  {/if}
  {#if item.isFavorite}
    <span class="favorite">★</span>
  {/if}
  {#if item.hasChildren}
    <span class="chevron">›</span>
  {/if}
</div>

<style>
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
    font-size: var(--text-base);
    color: var(--color-text);
    transition: background 0.08s;
  }

  .column-item:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .column-item:hover {
    background: var(--color-hover);
  }

  .column-item.selected {
    background: var(--color-accent);
    color: white;
  }

  .column-item.inactive {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .column-item.workspace-selected {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .column-item.workspace-selected.selected {
    outline-color: white;
  }

  .icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    opacity: 0.7;
  }

  .column-item.selected:not(.inactive) .icon {
    opacity: 1;
  }

  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .git {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    max-width: 96px;
    font-size: var(--text-2xs);
    color: var(--color-text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .git .branch {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.75;
  }

  .git .dirty {
    color: #e0a030;
    font-size: 0.55rem;
    line-height: 1;
  }

  .git .sync {
    opacity: 0.75;
  }

  .column-item.selected:not(.inactive) .git,
  .column-item.selected:not(.inactive) .git .dirty {
    color: white;
  }

  .chevron {
    flex-shrink: 0;
    opacity: 0.4;
    font-size: var(--text-lg);
    line-height: 1;
  }

  .column-item.selected:not(.inactive) .chevron {
    opacity: 0.8;
  }

  .favorite {
    flex-shrink: 0;
    font-size: var(--text-2xs);
    opacity: 0.6;
    color: var(--color-accent);
  }

  .column-item.selected:not(.inactive) .favorite {
    color: white;
    opacity: 0.9;
  }
</style>
