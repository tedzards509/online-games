<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import { t } from '../../lib/i18n';
  import PlayingCard from './PlayingCard.svelte';

  /** Face-down card showing the theme's back. Never pass the concealed face. */
  let {
    label,
    background = 'var(--arena-card-back)',
    ...shell
  }: Omit<ComponentProps<typeof PlayingCard>, 'children' | 'label'> & {
    label?: string;
  } = $props();
</script>

<PlayingCard {...shell} {background} label={label ?? $t('hiddenCard')}>
  <span></span>
</PlayingCard>
