<script lang="ts">
  /** A card-sized placeholder: a dashed outline (e.g. an empty pile), or an
   * invisible gap that keeps a layout slot (`outlined={false}`). */
  let {
    label,
    width,
    outlined = true,
  }: { label: string; width?: string; outlined?: boolean } = $props();
</script>

<div
  class="slot"
  class:outlined
  role="img"
  aria-label={label}
  style:width
></div>

<style>
  .slot {
    flex: 0 0 auto;
    width: var(--arena-card-width);
    aspect-ratio: 5 / 7;
    border-radius: var(--arena-card-radius);
  }
  .outlined {
    border: var(--arena-line);
    border-style: dashed;
  }
</style>
