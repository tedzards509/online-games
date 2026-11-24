<script lang="ts">
  /** Decorative initial in one of the theme's seat colors (usually the player
   * index); always render the name beside it. */
  let { name, colorIndex }: { name: string; colorIndex: number } = $props();
  const seatColors = 6;
  const initial = $derived([...name.trim()][0]?.toUpperCase() ?? '?');
</script>

<span
  class="avatar"
  aria-hidden="true"
  style:background="var(--arena-seat-{(colorIndex % seatColors) + 1})"
  >{initial}</span
>

<style>
  .avatar {
    flex: none;
    display: grid;
    place-items: center;
    width: 2.25rem;
    height: 2.25rem;
    border: var(--arena-line);
    border-radius: var(--arena-avatar-radius);
    color: var(--arena-on-seat);
    font-weight: 800;
  }
</style>
