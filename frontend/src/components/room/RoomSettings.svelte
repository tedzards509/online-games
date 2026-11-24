<script lang="ts">
  import { t } from '../../lib/i18n';
  import { games, type GameKind, type Schemas } from '../../lib/types';
  import { crazyEightsDefaults, crazyEightsRules } from '../../lib/config';
  import Checkbox from '../ui/Checkbox.svelte';
  import ChoiceGroup from '../ui/ChoiceGroup.svelte';
  import Select from '../ui/Select.svelte';

  /** Host controls for the selected game and its rules. */
  let {
    config,
    disabled,
    change,
    players,
  }: {
    config: Schemas['Config'] | null | undefined;
    disabled: boolean;
    change: (config: Schemas['Config']) => void;
    players: string[];
  } = $props();

  function select(game: GameKind): void {
    change(
      game === 'crazy_eights'
        ? { game, rules: { ...crazyEightsDefaults } }
        : game === 'shithead'
          ? { game, rules: { four_card_bombs: false } }
          : game === 'tempel_des_schreckens'
            ? { game, rules: { initial_key: 0 } }
            : { game, rules: {} },
    );
  }
</script>

<ChoiceGroup
  legend={$t('room.game')}
  options={games.map((game) => ({
    value: game,
    label: $t(`gameNames.${game}`),
  }))}
  value={config?.game}
  {disabled}
  onchange={select}
/>
{#if config?.game === 'crazy_eights'}
  <fieldset {disabled}>
    <legend>{$t('room.gameRules')}</legend>
    {#each crazyEightsRules as rule (rule)}
      <Checkbox
        label={$t(`crazyEights.rules.${rule}`)}
        {disabled}
        checked={config.rules[rule]}
        onchange={(checked) => {
          if (config?.game === 'crazy_eights')
            change({ ...config, rules: { ...config.rules, [rule]: checked } });
        }}
      />
    {/each}
  </fieldset>
{:else if config?.game === 'shithead'}
  <Checkbox
    label={$t('shithead.rules.bombs')}
    {disabled}
    checked={config.rules.four_card_bombs}
    onchange={(checked) =>
      change({ game: 'shithead', rules: { four_card_bombs: checked } })}
  />
{:else if config?.game === 'tempel_des_schreckens'}
  <Select
    label={$t('room.rules.initialKey')}
    {disabled}
    value={config.rules.initial_key}
    options={(players.length
      ? players
      : [$t('room.playerNumber', { values: { number: 1 } })]
    ).map((name, index) => ({
      value: index,
      label: name,
    }))}
    onchange={(initial_key) =>
      change({ game: 'tempel_des_schreckens', rules: { initial_key } })}
  />
{/if}

<style>
  fieldset {
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: var(--arena-space-1);
    padding: 0;
    font-weight: 800;
  }
</style>
