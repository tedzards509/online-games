<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Inline message. Errors are announced as alerts, other notices as status. */
  let {
    tone = 'info',
    children,
  }: { tone?: 'info' | 'error'; children: Snippet } = $props();
</script>

<p class={tone} role={tone === 'error' ? 'alert' : 'status'}>
  {@render children()}
</p>

<style>
  p {
    margin: 0;
    padding: 0.6rem 1rem;
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    font-weight: 600;
  }
  .error {
    background: var(--arena-error-surface);
    color: var(--arena-error);
  }
</style>
