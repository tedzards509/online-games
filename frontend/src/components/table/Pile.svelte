<script lang="ts">
  import type { Snippet } from 'svelte';

  /** A card pile on the table with a short caption, e.g. "18 in stock". */
  let { caption, children }: { caption: string; children: Snippet } = $props();
</script>

<figure>
  {@render children()}
  <figcaption>{caption}</figcaption>
</figure>

<style>
  figure {
    display: grid;
    justify-items: center;
    gap: var(--arena-space-1);
    margin: 0;
  }
  figcaption {
    font-size: 0.9rem;
    font-weight: 700;
  }
</style>
