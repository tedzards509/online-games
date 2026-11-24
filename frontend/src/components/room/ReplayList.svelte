<script lang="ts">
  import { games, type Snapshot } from '../../lib/types';
  import Button from '../ui/Button.svelte';

  /** Completed games of this room, downloadable by every member. */
  let {
    replays,
    download,
  }: {
    replays: Snapshot['replays'];
    download: (id: string) => void;
  } = $props();
</script>

{#if replays.length}
  <details>
    <summary>Replays ({replays.length})</summary>
    <div class="list">
      {#each replays as archive, index (archive.id)}
        <Button
          variant="quiet"
          size="small"
          onclick={() => download(archive.id)}
          >Game {index + 1} · {games[archive.config.game]} ↓</Button
        >
      {/each}
    </div>
  </details>
{/if}

<style>
  summary {
    min-height: 2.75rem;
    display: flex;
    align-items: center;
    font-weight: 800;
    cursor: pointer;
  }
  .list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    padding: var(--arena-space-1) 0 0.5rem;
  }
</style>
