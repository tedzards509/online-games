<script lang="ts">
  import PlayingCard from '../../src/components/cards/PlayingCard.svelte';
  import SymbolCard from '../../src/components/cards/SymbolCard.svelte';
  import StandardPlayingCard from '../../src/components/cards/StandardPlayingCard.svelte';
  import CardBack from '../../src/components/cards/CardBack.svelte';
  import { setLanguage } from '../../src/lib/i18n';
  let selected = $state(false);
</script>

<main>
  <PlayingCard label="Custom moon" width="100px">
    <svg viewBox="0 0 32 32" width="80%"
      ><circle cx="16" cy="16" r="12" fill="gold" /></svg
    >
  </PlayingCard>
  <SymbolCard
    label="Purple star"
    rank="X"
    symbol="★"
    color="rebeccapurple"
    background="lavender"
    width="100px"
  />
  <StandardPlayingCard card={{ rank: 14, suit: 'hearts' }} width="100px" />
  <StandardPlayingCard card={{ rank: 10, suit: 'spades' }} width="100px" />
  <StandardPlayingCard card={{ rank: 12, suit: 'diamonds' }} width="100px" />
  <StandardPlayingCard
    card={{ rank: 13, suit: 'clubs' }}
    width="100px"
    onclick={() => (selected = !selected)}
    {selected}
  />
  <SymbolCard
    label="Disabled card"
    rank="2"
    symbol="!"
    onclick={() => {
      throw new Error('Disabled card clicked');
    }}
    disabled
  />
  <CardBack width="100px" />
  <button onclick={() => setLanguage('de-cool')}>German</button>
</main>
