<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  /** Text input whose label is both its placeholder and accessible name. */
  let {
    label,
    value = $bindable(''),
    ...attributes
  }: Omit<HTMLInputAttributes, 'value'> & {
    label: string;
    value?: string;
  } = $props();
</script>

<input {...attributes} bind:value aria-label={label} placeholder={label} />

<style>
  input {
    width: 100%;
    min-width: 0;
    min-height: var(--arena-control-height);
    padding: 0.5rem 1rem;
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    background: var(--arena-input);
    font-size: 1.05rem;
    font-weight: 600;
  }
  input::placeholder {
    color: var(--arena-text-muted);
    opacity: 1;
  }
</style>
