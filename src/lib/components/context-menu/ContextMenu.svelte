<script lang="ts">
  import { contextMenuStore } from '$lib/stores/contextMenu.svelte';

  let menuEl = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (contextMenuStore.visible && menuEl) {
      const menuWidth = 200;
      const menuHeight = menuEl.offsetHeight;

      let adjustedX = contextMenuStore.x;
      let adjustedY = contextMenuStore.y;

      if (adjustedX + menuWidth > window.innerWidth) {
        adjustedX = window.innerWidth - menuWidth - 8;
      }
      if (adjustedY + menuHeight > window.innerHeight) {
        adjustedY = window.innerHeight - menuHeight - 8;
      }

      menuEl.style.left = `${adjustedX}px`;
      menuEl.style.top = `${adjustedY}px`;
    }
  });

  $effect(() => {
    if (!contextMenuStore.visible) return;

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        contextMenuStore.hide();
      }
    }

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if contextMenuStore.visible}
  <div
    class="cm-backdrop"
    onclick={() => contextMenuStore.hide()}
    role="presentation"
  />
  <div
    class="cm-menu"
    style="left: {contextMenuStore.x}px; top: {contextMenuStore.y}px"
    bind:this={menuEl}
    role="menu"
  >
    {#each contextMenuStore.items as item}
      {#if item.divider}
        <div class="cm-divider" role="separator" />
      {:else}
        <button
          class="cm-item"
          class:danger={item.danger}
          class:disabled={item.disabled}
          role="menuitem"
          onclick={() => {
            if (!item.disabled) {
              item.action();
              contextMenuStore.hide();
            }
          }}
        >
          {item.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .cm-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
  }

  .cm-menu {
    position: fixed;
    z-index: 1000;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.2);
    padding: 4px;
    min-width: 180px;
  }

  .cm-item {
    display: block;
    width: 100%;
    padding: 6px 12px;
    text-align: left;
    border: none;
    background: none;
    color: var(--color-text);
    font-size: 0.875rem;
    cursor: pointer;
    border-radius: 5px;
  }

  .cm-item:hover {
    background: var(--color-hover);
  }

  .cm-item.danger {
    color: #e53e3e;
  }

  .cm-item.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .cm-divider {
    height: 1px;
    background: var(--color-border);
    margin: 3px 8px;
  }
</style>
