<script lang="ts">
  import type { Action, BoardProps, Schemas } from '../lib/types';
  import CardBack from './cards/CardBack.svelte';
  import CardSlot from './cards/CardSlot.svelte';
  import PlayingCard from './cards/PlayingCard.svelte';
  import Actions from './table/Actions.svelte';
  import Board from './table/Board.svelte';
  import Pile from './table/Pile.svelte';
  import PlayerBar from './table/PlayerBar.svelte';
  import TableSurface from './table/TableSurface.svelte';
  import Button from './ui/Button.svelte';
  import Notice from './ui/Notice.svelte';

  let {
    view,
    actions,
    you,
    names,
    active,
    move,
  }: BoardProps<Schemas['SkyjoView']> = $props();

  /** Printed Skyjo value bands, each with a contrasting ink. */
  function bandColors(value: number): { face: string; ink: string } {
    if (value < 0) return { face: '#34479a', ink: '#fff' };
    if (value === 0) return { face: '#8fd3f4', ink: '#16271e' };
    if (value <= 4) return { face: '#7cc472', ink: '#16271e' };
    if (value <= 8) return { face: '#f6d44c', ink: '#16271e' };
    return { face: '#c63d29', ink: '#fff' };
  }

  const order = $derived([
    you,
    ...names.map((_, i) => i).filter((i) => i !== you),
  ]);
  function cellAction(player: number, index: number): Action | undefined {
    if (player !== you) return;
    return actions.find(
      (a) =>
        a.game === 'skyjo' &&
        (a.action.type === 'reveal' || a.action.type === 'replace') &&
        a.action.index === index,
    );
  }
  const hint = $derived(
    view.phase === 'setup'
      ? 'Reveal two'
      : view.phase === 'place'
        ? 'Replace a card'
        : view.phase === 'reveal'
          ? 'Reveal a card'
          : 'Draw a card',
  );
</script>

{#snippet face(value: number, label: string, action?: Action)}
  {@const colors = bandColors(value)}
  <PlayingCard
    {label}
    color={colors.ink}
    background={colors.face}
    onclick={action && (() => move(action))}
  >
    <span class="value">{value}</span>
  </PlayingCard>
{/snippet}

<Board>
  <TableSurface>
    <Pile caption={`${view.stock} in stock`}><CardBack label="Stock" /></Pile>
    <Pile caption="Discard">
      {#if view.top != null}{@render face(
          view.top,
          'Discard pile',
        )}{:else}<CardSlot label="Discard pile" />{/if}
    </Pile>
    {#if view.pending != null}
      <Pile caption="Drawn">{@render face(view.pending, 'Drawn card')}</Pile>
    {/if}
  </TableSurface>

  {#if actions.length || view.phase === 'setup'}
    <Notice>{actions.length ? hint : 'Waiting for other players'}</Notice>
  {/if}
  <Actions>
    {#each actions as action}
      {#if action.game === 'skyjo' && action.action.type === 'draw'}
        <Button onclick={() => move(action)}
          >{action.action.source === 'stock' ? 'Draw' : 'Take discard'}</Button
        >
      {:else if action.game === 'skyjo' && action.action.type === 'discard'}
        <Button variant="secondary" onclick={() => move(action)}
          >Discard & reveal</Button
        >
      {/if}
    {/each}
  </Actions>

  <div class="grids">
    {#each order as player (player)}
      <section
        class="player"
        class:own={player === you}
        aria-label={`${names[player]}'s grid`}
      >
        <PlayerBar
          name={names[player]}
          colorIndex={player}
          you={player === you}
          active={active.includes(player)}
        />
        <div class="grid">
          {#each view.grids[player] as cell, index}
            {@const action = cellAction(player, index)}
            {@const label = `Card ${index + 1}: ${cell.removed ? 'removed' : (cell.value ?? 'hidden')}`}
            {#if cell.removed}
              <CardSlot {label} outlined={false} />
            {:else if cell.value == null}
              <CardBack {label} onclick={action && (() => move(action))} />
            {:else}
              {@render face(cell.value, label, action)}
            {/if}
          {/each}
        </div>
      </section>
    {/each}
  </div>
</Board>

<style>
  .value {
    font-size: 45cqw;
    font-weight: 800;
  }
  .grids {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--arena-space-3) var(--arena-space-4);
  }
  .player {
    display: grid;
    justify-items: start;
    gap: var(--arena-space-1);
    --arena-card-width: var(--arena-card-width-small);
  }
  .own {
    flex-basis: 100%;
    justify-items: center;
    --arena-card-width: 4.5rem;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(4, var(--arena-card-width));
    gap: 0.5rem;
  }
</style>
