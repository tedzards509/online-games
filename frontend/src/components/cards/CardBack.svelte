<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import { t } from '../../lib/i18n';
  import PlayingCard from './PlayingCard.svelte';

  /** Face-down card: never pass the concealed face as a prop or snippet. */
  let {
    label,
    background = 'var(--arena-card-back)',
    ...shell
  }: Omit<ComponentProps<typeof PlayingCard>, 'children' | 'label'> & {
    label?: string;
  } = $props();
</script>

<PlayingCard {...shell} {background} label={label ?? $t('hiddenCard')}>
  <span class="back-pattern"></span>
</PlayingCard>

<style>
  .back-pattern {
    position: absolute;
    inset: 7%;
    border: 1px solid var(--arena-card-face);
    border-radius: calc(var(--arena-card-radius) / 2);
    background: repeating-linear-gradient(
      45deg,
      transparent 0 7px,
      #fff8e725 7px 8px
    );
  }
</style>
