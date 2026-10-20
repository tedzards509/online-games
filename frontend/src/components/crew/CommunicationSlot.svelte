<script lang="ts">
  import type { Schemas } from '../../lib/types';
  import { t } from '../../lib/i18n';
  import CrewCard from './CrewCard.svelte';
  let {
    communication,
    played,
    name,
  }: {
    communication: Schemas['CrewCommunication'] | null;
    played: boolean;
    name: string;
  } = $props();
  const status = $derived(
    $t(
      played
        ? 'crew.communicationPlayed'
        : communication
          ? `crew.declarations.${communication.declaration}`
          : 'crew.communicationReady',
    ),
  );
</script>

<div
  class="communication"
  role="group"
  aria-label={$t('crew.communicationOf', { values: { name } })}
  data-state={played ? 'played' : communication ? 'communicated' : 'ready'}
>
  <div class="slot">
    {#if communication && !played}<CrewCard
        card={communication.card}
        width="var(--crew-small-card)"
      />{/if}
  </div>
  <span
    class="marker"
    class:played
    data-position={communication?.declaration ?? 'highest'}
    role="img"
    aria-label={status}
    title={status}
  ></span>
</div>

<style>
  .communication {
    position: relative;
    width: var(--crew-small-card);
  }
  .slot {
    width: var(--crew-small-card);
    aspect-ratio: 5 / 7;
    border: 1px dashed var(--arena-border);
    border-radius: 0.5rem;
  }
  .slot:has(:global(.playing-card)) {
    border: 0;
  }
  .marker {
    position: absolute;
    right: -0.35rem;
    width: 0.875rem;
    height: 0.875rem;
    border: 2px solid var(--arena-card-face);
    border-radius: 50%;
    background: var(--arena-primary);
    box-shadow: 0 0 0 1px var(--arena-border);
  }
  .marker[data-position='highest'] {
    top: -0.3rem;
  }
  .marker[data-position='only'] {
    top: calc(50% - 0.4375rem);
  }
  .marker[data-position='lowest'] {
    bottom: -0.3rem;
  }
  .marker.played {
    background: var(--arena-card-red);
  }
</style>
