<script lang="ts">
  import { cardLabel, type Card } from '../lib/types';
  let {
    card,
    disabled = false,
    selected = false,
    onclick,
    label,
  }: {
    card?: Card | null;
    disabled?: boolean;
    selected?: boolean;
    onclick?: () => void;
    label?: string;
  } = $props();
</script>

<button
  class="card"
  class:red={card?.suit === 'hearts' || card?.suit === 'diamonds'}
  class:back={!card}
  class:selected
  {disabled}
  {onclick}
  aria-label={label ?? (card ? cardLabel(card) : 'Face-down card')}
  aria-pressed={onclick ? selected : undefined}
  >{card ? cardLabel(card) : '?'}</button
>
