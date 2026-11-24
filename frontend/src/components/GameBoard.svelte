<script lang="ts">
  import type { Action, GameSnapshot, Snapshot } from '../lib/types';
  import { t } from '../lib/i18n';
  import Notice from './ui/Notice.svelte';
  import Mancala from './Mancala.svelte';
  import CrazyEights from './CrazyEights.svelte';
  import Shithead from './Shithead.svelte';
  import Skyjo from './Skyjo.svelte';
  import TicTacToe from './TicTacToe.svelte';
  import TheCrew from './TheCrew.svelte';
  import TempelDesSchreckens from './TempelDesSchreckens.svelte';

  /** Routes the viewer's projection to its board; benched members see no board. */
  let {
    room,
    disabled,
    move,
    roomInfo,
  }: {
    room: GameSnapshot;
    disabled: boolean;
    move: (action: Action) => void;
    roomInfo?: Pick<Snapshot, 'code' | 'host'>;
  } = $props();
</script>

{#if room.you == null}
  <div class="bench">
    <Notice>{$t('room.onBench')}</Notice>
    {#if room.config.game === 'the_crew' && room.outcome}<p role="status">
        {$t(room.outcome.winners.length ? 'common.won' : 'common.lost')}
      </p>{/if}
  </div>
{:else}
  {@const common = {
    actions: disabled ? [] : room.legal_actions,
    names: room.players.map((player) => player.nickname),
    you: room.you,
    active: room.outcome ? [] : room.turn.players,
    finished: room.outcome !== null,
    move,
  }}
  {#if room.view?.game === 'tempel_des_schreckens'}
    <TempelDesSchreckens view={room.view.state} {...common} />
  {:else if room.view?.game === 'the_crew'}
    {#key room.id}
      <TheCrew
        view={room.view.state}
        {...common}
        actions={room.legal_actions}
        players={room.players}
        {disabled}
        {roomInfo}
        won={!!room.outcome?.winners.length}
      />
    {/key}
  {:else if room.view?.game === 'mancala'}
    <Mancala view={room.view.state} {...common} />
  {:else if room.view?.game === 'crazy_eights'}
    <CrazyEights view={room.view.state} {...common} />
  {:else if room.view?.game === 'shithead'}
    <Shithead view={room.view.state} {...common} />
  {:else if room.view?.game === 'skyjo'}
    <Skyjo view={room.view.state} {...common} />
  {:else if room.view?.game === 'tic_tac_toe'}
    <TicTacToe view={room.view.state} {...common} />
  {/if}
{/if}

<style>
  .bench {
    display: grid;
    place-content: center;
    justify-items: center;
    gap: var(--arena-space-2);
    text-align: center;
  }
</style>
