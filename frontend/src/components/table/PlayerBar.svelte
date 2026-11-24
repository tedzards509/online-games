<script lang="ts">
  import type { Snippet } from 'svelte';
  import Avatar from './Avatar.svelte';

  /** A player's identity strip: avatar, name, and optional public details.
   * `active` highlights a player who may act now (also exposed as data-active). */
  let {
    name,
    colorIndex,
    you = false,
    active = false,
    children,
  }: {
    name: string;
    colorIndex: number;
    you?: boolean;
    active?: boolean;
    children?: Snippet;
  } = $props();
</script>

<div class="player-bar" class:active data-active={active}>
  <Avatar {name} {colorIndex} />
  <span class="name"
    >{name}{#if you}<span class="you">{' · you'}</span>{/if}</span
  >
  {#if children}<span class="details">{@render children()}</span>{/if}
</div>

<style>
  .player-bar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem 0.65rem;
    min-width: 0;
    padding: 0.4rem 0.75rem 0.4rem 0.4rem;
    border: var(--arena-line);
    border-color: transparent;
    border-radius: var(--arena-control-radius);
    font-weight: 700;
  }
  .active {
    border-color: var(--arena-ink);
    background: var(--arena-highlight);
    color: var(--arena-on-highlight);
  }
  .name {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .you {
    font-weight: 500;
  }
  .details {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem 0.75rem;
    margin-left: auto;
    font-weight: 500;
  }
</style>
