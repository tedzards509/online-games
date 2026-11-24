<!-- Crown marking the room host. -->
<script lang="ts">
  import { t } from '../../lib/i18n';
</script>

<svg viewBox="0 0 24 24" role="img" aria-label={$t('room.host')}>
  <path d="M3 7l4.5 4L12 4l4.5 7L21 7l-2 12H5z" />
</svg>

<style>
  svg {
    flex: none;
    width: 1.3rem;
    height: 1.3rem;
    fill: var(--arena-secondary);
    stroke: var(--arena-ink);
    stroke-width: 2;
    stroke-linejoin: round;
  }
</style>
