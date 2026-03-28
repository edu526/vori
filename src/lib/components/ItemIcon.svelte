<script lang="ts">
  import type { NavItemType } from '$lib/stores/navigation.svelte';

  let { type, size = 14 }: { type: NavItemType | 'recent-project' | 'recent-file'; size?: number } = $props();
</script>

{#if type === 'category'}
  <!-- Folder with tab -->
  <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
    <path d="M1.5 3A1.5 1.5 0 0 1 3 1.5h3.879a1.5 1.5 0 0 1 1.06.44l1.122 1.12H13A1.5 1.5 0 0 1 14.5 4.5v8A1.5 1.5 0 0 1 13 14H3A1.5 1.5 0 0 1 1.5 12.5V3z"/>
  </svg>

{:else if type === 'project' || type === 'recent-project'}
  <!-- Package/box -->
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M13.5 5 8 2 2.5 5v6L8 14l5.5-3V5z"/>
    <polyline points="2.5 5 8 8 13.5 5"/>
    <line x1="8" y1="8" x2="8" y2="14"/>
  </svg>

{:else if type === 'file' || type === 'recent-file'}
  <!-- Document with text lines -->
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M9.5 1.5H4A1.5 1.5 0 0 0 2.5 3v10A1.5 1.5 0 0 0 4 14.5h8A1.5 1.5 0 0 0 13.5 13V5.5L9.5 1.5z"/>
    <polyline points="9.5 1.5 9.5 5.5 13.5 5.5"/>
    <line x1="5.5" y1="8.5" x2="10.5" y2="8.5"/>
    <line x1="5.5" y1="11" x2="8.5" y2="11"/>
  </svg>
{/if}
