<script lang="ts">
  import type { Schemas } from '../lib/types';
  import { crazyEightsRules } from '../lib/config';

  let { config }: { config: Schemas['Config'] } = $props();
</script>

<section aria-label="Game rules">
  {#if config.game === 'crazy_eights'}
    <ul>
      {#each Object.entries(crazyEightsRules) as [key, label]}
        <li>
          {label}: {config.rules[key as keyof typeof config.rules]
            ? 'On'
            : 'Off'}
        </li>
      {/each}
    </ul>
  {:else}
    <p class="muted">Standard rules</p>
  {/if}
</section>
