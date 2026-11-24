<script lang="ts">
  import { games, type GameSnapshot } from '../../lib/types';

  /** Game title with the final result, or whose turn it is when `showTurn`.
   * The strip is highlighted while the viewer may act. */
  let { game, showTurn = true }: { game: GameSnapshot; showTurn?: boolean } =
    $props();
  const names = $derived(game.players.map((player) => player.nickname));
  const yourTurn = $derived(
    showTurn &&
      !game.outcome &&
      game.you != null &&
      game.turn.players.includes(game.you),
  );
  const message = $derived.by(() => {
    if (game.outcome)
      return game.outcome.winners.length
        ? `${game.outcome.winners.map((player) => names[player]).join(', ')} won`
        : game.config.game === 'the_crew'
          ? 'Lost'
          : 'Draw';
    if (!showTurn || !game.turn.players.length) return '';
    return yourTurn
      ? 'Your turn'
      : `${game.turn.players.map((player) => names[player]).join(', ')}'s turn`;
  });
</script>

<div class="game-status" class:your-turn={yourTurn}>
  <h1>{games[game.config.game]}</h1>
  {#if message}<p role="status">{message}</p>{/if}
  {#if game.outcome?.scores}<p>
      {game.outcome.scores
        .map((score, player) => `${names[player]}: ${score}`)
        .join(' · ')}
    </p>{/if}
</div>

<style>
  .game-status {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.25rem var(--arena-space-3);
    padding: 0.6rem 1rem;
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    background: var(--arena-surface);
  }
  .your-turn {
    background: var(--arena-highlight);
    color: var(--arena-on-highlight);
  }
  [role='status'] {
    font-size: 1.1rem;
    font-weight: 800;
  }
</style>
