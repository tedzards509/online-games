<script lang="ts">
  import { games, type GameKind, type Schemas } from '../lib/types';
  import { crazyEightsDefaults, crazyEightsRules } from '../lib/config';
  let {
    config,
    disabled,
    change,
  }: {
    config: Schemas['Config'] | null | undefined;
    disabled: boolean;
    change: (config: Schemas['Config'] | null) => void;
  } = $props();
  function select(value: string) {
    if (!value) return change(null);
    const game = value as GameKind;
    change(
      game === 'crazy_eights'
        ? { game, rules: { ...crazyEightsDefaults } }
        : game === 'shithead'
          ? { game, rules: { four_card_bombs: false } }
          : { game, rules: {} },
    );
  }
</script>

<label
  >Game<select
    value={config?.game ?? ''}
    {disabled}
    onchange={(e) => select(e.currentTarget.value)}
  >
    <option value="">Choose a game</option>
    {#each Object.entries(games) as [value, name]}<option {value}>{name}</option
      >{/each}
  </select></label
>
{#if config?.game === 'crazy_eights'}
  <fieldset {disabled}>
    <legend>Game rules</legend>
    {#each Object.entries(crazyEightsRules) as [key, label]}
      <label
        ><input
          type="checkbox"
          checked={config.rules[key as keyof typeof config.rules]}
          onchange={(e) => {
            if (config?.game === 'crazy_eights')
              change({
                ...config,
                rules: { ...config.rules, [key]: e.currentTarget.checked },
              });
          }}
        />{label}</label
      >
    {/each}
  </fieldset>
{:else if config?.game === 'shithead'}
  <label
    ><input
      type="checkbox"
      {disabled}
      checked={config.rules.four_card_bombs}
      onchange={(e) =>
        change({
          game: 'shithead',
          rules: { four_card_bombs: e.currentTarget.checked },
        })}
    />Four-card bombs bypass rank restrictions</label
  >
{/if}

<style>
  fieldset {
    min-width: 0;
  }
  fieldset label {
    display: block;
  }
  input[type='checkbox'] {
    width: auto;
  }
</style>
