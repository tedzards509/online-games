<script lang="ts">
  import type { Action, GameSnapshot, Snapshot } from '../lib/types';
  import Mancala from './Mancala.svelte';
  import CrazyEights from './CrazyEights.svelte';
  import Shithead from './Shithead.svelte';
  import Skyjo from './Skyjo.svelte';
  import TicTacToe from './TicTacToe.svelte';
  import TheCrew from './TheCrew.svelte';
  import TempelDesSchreckens from './TempelDesSchreckens.svelte';
  import { t } from '../lib/i18n';
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
  const actions = $derived(disabled ? [] : room.legal_actions);
  const names = $derived(room.players.map((p) => p.nickname));
</script>

{#if room.you == null}
  <div class:crew-bench={room.config.game === 'the_crew'}>
    <p>You are on the bench</p>
    {#if room.config.game === 'the_crew' && room.outcome}<p role="status">
        {$t(room.outcome.winners.length ? 'crew.won' : 'crew.lost')}
      </p>{/if}
  </div>
{:else if room.view?.game === 'tempel_des_schreckens'}
  <TempelDesSchreckens
    view={room.view.state}
    {actions}
    {names}
    {move}
    you={room.you}
    finished={room.outcome !== null}
  />
{:else if room.view?.game === 'the_crew'}
  {#key room.id}
    <TheCrew
      view={room.view.state}
      actions={room.legal_actions}
      players={room.players}
      you={room.you}
      {disabled}
      {roomInfo}
      {move}
      finished={room.outcome !== null}
      won={!!room.outcome?.winners.length}
    />
  {/key}
{:else if room.view?.game === 'mancala'}
  <Mancala view={room.view.state} {actions} you={room.you} {move} />
{:else if room.view?.game === 'crazy_eights'}
  <div class="opponents">
    {#each room.players as player, index}{#if index !== room.you}<span
          >{player.nickname} · {room.view.state.counts[index]} cards</span
        >{/if}{/each}
  </div>
  <CrazyEights view={room.view.state} {actions} {move} />
{:else if room.view?.game === 'shithead'}
  <Shithead view={room.view.state} {actions} you={room.you} {names} {move} />
{:else if room.view?.game === 'skyjo'}
  <Skyjo view={room.view.state} {actions} you={room.you} {names} {move} />
{:else if room.view?.game === 'tic_tac_toe'}
  <TicTacToe view={room.view.state} {actions} you={room.you} {move} />
{/if}

<style>
  .crew-bench {
    display: grid;
    place-content: center;
    text-align: center;
  }
</style>
