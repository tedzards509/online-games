<script lang="ts">
  import { locale, t, languages, setLanguage } from '../lib/i18n';
</script>

<div class="settings" lang={$locale === 'de-cool' ? 'de' : $locale}>
  <button
    type="button"
    popovertarget="language-settings"
    aria-label={$t('settings')}
    title={$t('settings')}
  >
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      aria-hidden="true"
    >
      <path
        d="m9 3-.7 2.3-2 .9L4 5.6 2 9l1.7 1.7v2.6L2 15l2 3.4 2.3-.6 2 .9L9 21h4l.7-2.3 2-.9 2.3.6 2-3.4-1.7-1.7v-2.6L20 9l-2-3.4-2.3.6-2-.9L13 3Z"
        transform="translate(1)"
      />
      <circle cx="12" cy="12" r="3" />
    </svg>
  </button>
  <div id="language-settings" popover>
    <label for="language">{$t('language')}</label>
    <select
      id="language"
      value={$locale}
      onchange={(event) => setLanguage(event.currentTarget.value)}
    >
      {#each languages as language}
        <option
          value={language.id}
          lang={language.id === 'de-cool' ? 'de' : language.id}
          >{language.label}</option
        >
      {/each}
    </select>
  </div>
</div>

<style>
  .settings {
    position: fixed;
    top: var(--arena-space-3);
    right: max(var(--arena-space-3), env(safe-area-inset-right));
    z-index: 10;
    font-family: var(--arena-font);
    color: var(--arena-text);
  }
  button {
    display: grid;
    place-items: center;
    width: var(--arena-control-height);
    height: var(--arena-control-height);
    padding: 0.75rem;
    border: 1px solid var(--arena-border);
    border-radius: 50%;
    background: var(--arena-surface);
    color: var(--arena-text);
  }
  button:hover:not(:disabled) {
    background: var(--arena-input);
    border-color: var(--arena-text);
  }
  svg {
    width: 100%;
    height: 100%;
  }
  [popover] {
    position: fixed;
    inset: 5.25rem var(--arena-space-3) auto auto;
    margin: 0;
    width: min(17rem, calc(100vw - 3rem));
    padding: var(--arena-space-2);
    border: 1px solid var(--arena-border);
    border-radius: var(--arena-control-radius);
    box-shadow: var(--arena-shadow);
    background: var(--arena-surface);
    color: var(--arena-text);
    font-family: var(--arena-font);
  }
  label {
    margin-bottom: var(--arena-space-1);
    font-weight: 700;
  }
  select {
    min-height: var(--arena-control-height);
    padding: 0.5rem;
    border-color: var(--arena-border);
    border-radius: var(--arena-control-radius);
    background: var(--arena-input);
    font: inherit;
  }
  button:focus-visible,
  select:focus-visible {
    outline: 3px solid var(--arena-focus);
    outline-offset: 3px;
  }
  @media (max-width: 40rem) {
    .settings {
      top: auto;
      right: max(var(--arena-space-2), env(safe-area-inset-right));
      bottom: max(var(--arena-space-2), env(safe-area-inset-bottom));
    }
    [popover] {
      inset: auto var(--arena-space-2) calc(5rem + env(safe-area-inset-bottom))
        auto;
    }
  }
</style>
