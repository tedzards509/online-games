<script lang="ts">
  import type { Action, Snapshot } from '../lib/types';
  import Mancala from './Mancala.svelte';
  import CrazyEights from './CrazyEights.svelte';
  import Shithead from './Shithead.svelte';
  import Skyjo from './Skyjo.svelte';
  import TicTacToe from './TicTacToe.svelte';
  let {
    room,
    disabled,
    move,
  }: { room: Snapshot; disabled: boolean; move: (action: Action) => void } =
    $props();
  const actions = $derived(disabled ? [] : room.legal_actions);
  const names = $derived(room.players.map((p) => p.nickname));
</script>

{#if room.view?.game === 'mancala'}
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
