<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../../lib/i18n';
  import CardBack from '../cards/CardBack.svelte';
  import Button from '../ui/Button.svelte';
  import Avatar from './Avatar.svelte';

  /** An opponent at a `CardTable`: avatar and name, a stack of card backs
   * for their hand size, and game-specific public `markers` below. With
   * `info`, the name opens a popover with those public details. */
  let {
    name,
    colorIndex,
    cards,
    active = false,
    markers,
    info,
  }: {
    name: string;
    colorIndex: number;
    /** Number of cards in their hand. */
    cards: number;
    /** It is this player's turn. */
    active?: boolean;
    markers?: Snippet;
    info?: Snippet;
  } = $props();
  const id = $props.id();
</script>

<section class="player" aria-label={name} class:active>
  <div class="identity">
    <Avatar {name} {colorIndex} />
    {#if info}<button
        class="name"
        popovertarget={id}
        aria-label={$t('common.infoAbout', { values: { name } })}
        ><span title={name}>{name}</span><svg
          viewBox="0 0 20 20"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          aria-hidden="true"
          ><circle cx="10" cy="10" r="8" /><path d="M10 9v5" /><circle
            cx="10"
            cy="6"
            r="0.6"
            fill="currentColor"
          /></svg
        ></button
      >{:else}<span class="name"><span title={name}>{name}</span></span>{/if}
  </div>
  <div class="backs" aria-hidden="true">
    {#each Array(cards) as _}<span class="back"><CardBack /></span>{/each}
  </div>
  {#if markers}<div class="markers">{@render markers()}</div>{/if}
</section>

{#if info}
  <section
    popover
    {id}
    class="info"
    aria-label={$t('common.infoAbout', { values: { name } })}
  >
    <h2>{name}</h2>
    {@render info()}
    <Button
      variant="quiet"
      size="small"
      popovertarget={id}
      popovertargetaction="hide">{$t('common.close')}</Button
    >
  </section>
{/if}

<style>
  .player {
    min-width: 0;
    text-align: center;
  }
  .identity {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.375rem;
    width: fit-content;
    max-width: 100%;
    margin-inline: auto;
    padding: 0.2rem;
    border: var(--arena-line);
    border-color: transparent;
    border-radius: var(--arena-control-radius);
  }
  /* Matches PlayerBar's active highlight, around avatar and name. */
  .active .identity {
    border-color: var(--arena-ink);
    background: var(--arena-highlight);
    color: var(--arena-on-highlight);
  }
  .name {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    min-width: 0;
    min-height: 2.75rem;
    padding: 0 0.25rem;
    border: 0;
    border-radius: var(--arena-control-radius);
    background: transparent;
    color: inherit;
    font-weight: 800;
    font-size: 0.875rem;
  }
  button.name {
    cursor: pointer;
  }
  .name > span {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .name svg {
    flex-shrink: 0;
    width: 1.125rem;
    height: 1.125rem;
  }
  .backs {
    display: flex;
    justify-content: center;
    min-height: calc(var(--arena-card-width) * 1.4);
  }
  .back {
    width: 3px;
    flex: 0 0 3px;
  }
  .back:last-child {
    width: var(--arena-card-width);
    flex-basis: var(--arena-card-width);
  }
  .markers {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    margin-top: 0.25rem;
  }
  .info {
    width: min(24rem, calc(100vw - 2rem));
    max-height: calc(100dvh - 2rem);
    padding: var(--arena-space-3);
    border: var(--arena-line);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    overflow-y: auto;
    box-shadow: var(--arena-overlay);
  }
  h2 {
    margin: 0 0 1rem;
    overflow-wrap: anywhere;
  }
  /* CardTable's compact portrait seats: the name goes below the avatar so it
     keeps the seat's full width. */
  @media (max-width: 40rem) and (orientation: portrait) {
    .identity {
      flex-direction: column;
      gap: 0;
      padding: 0;
    }
    .name {
      font-size: 0.8rem;
    }
  }
</style>
