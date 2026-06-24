<script lang="ts">
  import { onMount } from 'svelte';
  import { updaterStore } from '$lib/stores/updater.svelte';
  import { message } from '@tauri-apps/plugin-dialog';

  const currentVersion = __APP_VERSION__;
  let busy = $state(false);

  onMount(() => {
    // ponytail: silent check on launch. No UI feedback unless an update is found
    // or the user clicks the chip.
    updaterStore.refresh();
  });

  async function handleClick() {
    if (busy) return;
    busy = true;
    try {
      if (updaterStore.available) {
        await updaterStore.promptInstall();
        return;
      }
      await updaterStore.refresh();
      if (updaterStore.state === 'available') {
        await updaterStore.promptInstall();
      } else if (updaterStore.state === 'up-to-date') {
        await message(`You're on the latest version (v${currentVersion}).`, {
          title: 'Vori is up to date',
          kind: 'info',
        });
      } else if (updaterStore.state === 'error') {
        await message(`Could not check for updates.\n\n${updaterStore.lastError}`, {
          title: 'Update check failed',
          kind: 'error',
        });
      }
    } finally {
      busy = false;
    }
  }
</script>

<button
  class="version-chip"
  class:has-update={updaterStore.state === 'available'}
  class:is-checking={updaterStore.state === 'checking'}
  onclick={handleClick}
  disabled={busy}
  title={updaterStore.state === 'error' ? updaterStore.lastError : `Vori v${currentVersion}`}
>
  <span class="current">v{currentVersion}</span>
  {#if updaterStore.available}
    <span class="arrow" aria-hidden="true">→</span>
    <span class="latest">v{updaterStore.available.version}</span>
  {/if}
  {#if updaterStore.state === 'checking'}
    <span class="pulse" aria-hidden="true"></span>
  {/if}
</button>

<style>
  .version-chip {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    padding: 2px 8px;
    border: 1px solid transparent;
    border-radius: 999px;
    background: transparent;
    cursor: pointer;
    font-family: inherit;
    line-height: 1.2;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }

  .version-chip:hover:not(:disabled) {
    background: var(--color-hover);
    border-color: var(--color-border);
  }

  .version-chip:disabled {
    cursor: default;
  }

  .version-chip .current {
    font-size: 0.7rem;
    color: var(--color-text-secondary);
    opacity: 0.55;
    font-variant-numeric: tabular-nums;
  }

  .version-chip .arrow {
    font-size: 0.7rem;
    color: var(--color-text-secondary);
    opacity: 0.4;
  }

  .version-chip .latest {
    font-size: 0.7rem;
    color: var(--color-accent);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .version-chip.has-update {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
  }

  .version-chip.has-update .current {
    opacity: 0.7;
  }

  .pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-text-secondary);
    opacity: 0.6;
    animation: pulse 1.1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.2; transform: scale(0.85); }
    50% { opacity: 0.9; transform: scale(1); }
  }
</style>
