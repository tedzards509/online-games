<script lang="ts">
  /** Native checkbox with an outlined box and a visible label. */
  let {
    label,
    checked,
    disabled = false,
    onchange,
  }: {
    label: string;
    checked: boolean;
    disabled?: boolean;
    onchange: (checked: boolean) => void;
  } = $props();
</script>

<label class:disabled>
  <input
    type="checkbox"
    {checked}
    {disabled}
    onchange={(event) => onchange(event.currentTarget.checked)}
  />
  {label}
</label>

<style>
  label {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-height: 2.75rem;
    font-weight: 600;
    cursor: pointer;
  }
  label.disabled {
    color: var(--arena-text-muted);
    cursor: not-allowed;
  }
  input {
    appearance: none;
    flex: none;
    display: grid;
    place-content: center;
    width: 1.6rem;
    height: 1.6rem;
    margin: 0;
    border: var(--arena-line);
    border-radius: 0.4rem;
    background: var(--arena-input);
    cursor: inherit;
  }
  input:checked {
    background: var(--arena-primary);
  }
  input:checked::after {
    content: '';
    width: 0.45rem;
    height: 0.8rem;
    margin-top: -0.15rem;
    border: solid var(--arena-on-primary);
    border-width: 0 3px 3px 0;
    transform: rotate(45deg);
  }
  input:disabled {
    background: var(--arena-disabled);
  }
</style>
