<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '../../lib/i18n';
  import type { RoomConnection } from '../../lib/connection.svelte';
  import type { GameSnapshot, Schemas } from '../../lib/types';
  import Avatar from '../table/Avatar.svelte';
  import Button from '../ui/Button.svelte';

  /** Modal end screen for a finished game: the result, every player with their
   * score, and the host's next step. Escape or "View board" dismisses it. */
  let {
    game,
    outcome,
    host,
    connection,
    disabled,
    onclose,
  }: {
    game: GameSnapshot;
    outcome: Schemas['Outcome'];
    host: boolean;
    connection: RoomConnection;
    disabled: boolean;
    onclose: () => void;
  } = $props();
  let dialog: HTMLDialogElement;
  onMount(() => dialog.showModal());

  const names = $derived(game.players.map((player) => player.nickname));
  const result = $derived(
    outcome.winners.length
      ? $t('common.playerWon', {
          values: {
            name: outcome.winners.map((player) => names[player]).join(', '),
          },
        })
      : $t(game.config.game === 'the_crew' ? 'common.lost' : 'common.draw'),
  );
</script>

<dialog bind:this={dialog} aria-labelledby="game-over-result" {onclose}>
  <p class="game">{$t(`gameNames.${game.config.game}`)}</p>
  <h2 id="game-over-result">{result}</h2>
  <ol aria-label={$t('common.players')}>
    {#each names as name, player (player)}
      {@const winner = outcome.winners.includes(player)}
      <li class:winner>
        <Avatar {name} colorIndex={player} />
        <span class="name">
          {name}{player === game.you ? ` · ${$t('common.you')}` : ''}
        </span>
        {#if winner}<span class="tag">{$t('room.winner')}</span>{/if}
        {#if outcome.scores}<span class="score"
            >{$t('room.score', {
              values: { score: outcome.scores[player] },
            })}</span
          >{/if}
      </li>
    {/each}
  </ol>
  {#if host}
    <div class="actions">
      <Button size="large" {disabled} onclick={() => connection.start()}
        >{$t('room.playAgain')}</Button
      >
      <Button variant="secondary" {disabled} onclick={() => connection.abort()}
        >{$t('room.returnToSetup')}</Button
      >
    </div>
  {:else}
    <p class="waiting">{$t('room.waitingForHost')}</p>
  {/if}
  <Button variant="quiet" size="small" onclick={() => dialog.close()}
    >{$t('room.viewBoard')}</Button
  >
</dialog>

<style>
  dialog {
    display: grid;
    gap: var(--arena-space-3);
    justify-items: center;
    width: min(28rem, calc(100vw - 2rem));
    max-height: calc(100dvh - 2rem);
    overflow: auto;
    padding: var(--arena-space-4) var(--arena-space-3);
    border: var(--arena-line);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    box-shadow: var(--arena-overlay);
    text-align: center;
  }
  dialog:not([open]) {
    display: none;
  }
  dialog::backdrop {
    background: color-mix(in srgb, var(--arena-ink) 45%, transparent);
  }
  .game {
    color: var(--arena-text-muted);
    font-weight: 800;
  }
  h2 {
    font-size: 1.8rem;
  }
  ol {
    display: grid;
    gap: var(--arena-space-1);
    width: 100%;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.5rem 0.75rem;
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    font-weight: 700;
    text-align: start;
  }
  .winner {
    background: var(--arena-highlight);
    color: var(--arena-on-highlight);
  }
  .name {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .tag {
    font-weight: 800;
  }
  .score {
    font-variant-numeric: tabular-nums;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.75rem;
  }
  .waiting {
    color: var(--arena-text-muted);
    font-weight: 700;
  }
</style>
