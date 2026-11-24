<script lang="ts">
  import { t } from '../../lib/i18n';
  import type { Schemas } from '../../lib/types';
  import { crazyEightsRules } from '../../lib/config';

  /** Read-only summary of the selected game's rules. */
  let { config }: { config: Schemas['Config'] } = $props();
</script>

<section aria-label={$t('room.gameRules')}>
  {#if config.game === 'crazy_eights'}
    <ul>
      <li>{$t('room.rules.crazyEightsEight')}</li>
      {#each crazyEightsRules as rule (rule)}
        <li>
          {$t('room.rules.state', {
            values: {
              rule: $t(`crazyEights.rules.${rule}`),
              state: $t(
                config.rules[rule] ? 'room.rules.on' : 'room.rules.off',
              ),
            },
          })}
        </li>
      {/each}
    </ul>
  {:else if config.game === 'shithead'}
    <p>
      {$t('room.rules.state', {
        values: {
          rule: $t('shithead.rules.bombs'),
          state: $t(
            config.rules.four_card_bombs ? 'room.rules.on' : 'room.rules.off',
          ),
        },
      })}
    </p>
  {:else if config.game === 'tempel_des_schreckens'}
    <p>
      {$t('room.rules.state', {
        values: {
          rule: $t('room.rules.initialKey'),
          state: $t('room.playerNumber', {
            values: { number: config.rules.initial_key + 1 },
          }),
        },
      })}
    </p>
  {:else}
    <p>{$t('room.rules.standard')}</p>
  {/if}
</section>

<style>
  ul {
    margin: 0;
    padding-left: 1.25rem;
  }
</style>
