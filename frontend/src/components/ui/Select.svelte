<script lang="ts" generics="T extends string | number">
  /** Native select with a visible label. */
  let {
    label,
    options,
    value,
    disabled = false,
    onchange,
  }: {
    label: string;
    options: { value: T; label: string; lang?: string }[];
    value: T;
    disabled?: boolean;
    onchange: (value: T) => void;
  } = $props();
  const id = $props.id();
</script>

<div class="field">
  <label for={id}>{label}</label>
  <select
    {id}
    {disabled}
    onchange={(event) =>
      onchange(options[event.currentTarget.selectedIndex].value)}
  >
    {#each options as option (option.value)}
      <option
        value={option.value}
        lang={option.lang}
        selected={option.value === value}>{option.label}</option
      >
    {/each}
  </select>
</div>

<style>
  .field {
    display: grid;
    gap: var(--arena-space-1);
  }
  label {
    font-weight: 800;
  }
  select {
    width: 100%;
    min-height: var(--arena-control-height);
    padding: 0.5rem 0.75rem;
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    background: var(--arena-input);
    font-weight: 600;
  }
  select:disabled {
    background: var(--arena-disabled);
    color: var(--arena-on-disabled);
  }
</style>
