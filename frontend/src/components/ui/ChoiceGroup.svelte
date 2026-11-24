<script lang="ts" generics="T extends string | number">
  /** Native radio group drawn as raised tiles; the chosen tile stays pressed in. */
  let {
    legend,
    options,
    value,
    disabled = false,
    onchange,
  }: {
    legend: string;
    options: { value: T; label: string }[];
    value: T | null | undefined;
    disabled?: boolean;
    onchange: (value: T) => void;
  } = $props();
  const name = $props.id();
</script>

<fieldset {disabled}>
  <legend>{legend}</legend>
  <div class="tiles">
    {#each options as option (option.value)}
      <label class="tile">
        <input
          type="radio"
          {name}
          checked={option.value === value}
          onchange={() => onchange(option.value)}
        />
        {option.label}
      </label>
    {/each}
  </div>
</fieldset>

<style>
  fieldset {
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: var(--arena-space-1);
    padding: 0;
    font-weight: 800;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    gap: 0.75rem;
  }
  .tile {
    position: relative;
    display: flex;
    align-items: center;
    min-height: 3.5rem;
    padding: 0.5rem 0.9rem;
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    background: var(--arena-surface);
    font-weight: 700;
    cursor: pointer;
    box-shadow: var(--arena-raised);
    transition:
      transform var(--arena-transition),
      box-shadow var(--arena-transition);
  }
  /* The transparent radio covers its tile: it receives pointer, keyboard, and
     assistive input while the tile shows its state. */
  input {
    position: absolute;
    inset: 0;
    margin: 0;
    opacity: 0;
    cursor: inherit;
  }
  .tile:has(input:checked) {
    transform: var(--arena-pressed-shift);
    box-shadow: var(--arena-pressed);
    background: var(--arena-primary);
    color: var(--arena-on-primary);
  }
  .tile:has(input:focus-visible) {
    outline: 3px solid var(--arena-focus);
    outline-offset: 3px;
  }
  fieldset:disabled .tile {
    cursor: not-allowed;
  }
  fieldset:disabled .tile:not(:has(input:checked)) {
    box-shadow: none;
    background: var(--arena-disabled);
    color: var(--arena-on-disabled);
  }
</style>
