<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

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
    padding: 0.75rem 1rem;
    border: 1.5px solid var(--arena-border);
    border-radius: var(--arena-control-radius);
    background: var(--arena-input);
    color: var(--arena-text);
    font-family: var(--arena-font);
    font-size: 1rem;
    line-height: 1.3;
  }
  input::placeholder {
    color: var(--arena-placeholder);
    opacity: 1;
  }
  input:focus-visible {
    outline: 3px solid var(--arena-focus);
    outline-offset: 3px;
  }
</style>
