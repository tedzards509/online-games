<script lang="ts">
  import type { Schemas } from '../lib/types';
  import { crazyEightsRules } from '../lib/config';

  let { config }: { config: Schemas['Config'] } = $props();
</script>

<section aria-label="Game rules">
  {#if config.game === 'crazy_eights'}
    <ul>
      <li>8: Choose suit (always)</li>
      {#each Object.entries(crazyEightsRules) as [key, label]}
        <li>
          {label}: {config.rules[key as keyof typeof config.rules]
            ? 'On'
            : 'Off'}
        </li>
      {/each}
    </ul>
  {:else if config.game === 'shithead'}
    <p>Four-card bombs: {config.rules.four_card_bombs ? 'On' : 'Off'}</p>
  {:else if config.game === 'tempel_des_schreckens'}
    <p>Initial key: Player {config.rules.initial_key + 1}</p>
  {:else}
    <p class="muted">Standard rules</p>
  {/if}
</section>
