<script lang="ts">
  import { locale, t, languages, setLanguage } from '../lib/i18n';
  import Button from './ui/Button.svelte';
  import Corner from './ui/Corner.svelte';
  import Popover from './ui/Popover.svelte';
  import Select from './ui/Select.svelte';
</script>

<div class="settings" lang={$locale === 'de-cool' ? 'de' : $locale}>
  <Corner placement="end">
    <Button
      variant="quiet"
      square
      popovertarget="language-settings"
      aria-label={$t('settings')}
      title={$t('settings')}
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        aria-hidden="true"
      >
        <path
          d="m9 3-.7 2.3-2 .9L4 5.6 2 9l1.7 1.7v2.6L2 15l2 3.4 2.3-.6 2 .9L9 21h4l.7-2.3 2-.9 2.3.6 2-3.4-1.7-1.7v-2.6L20 9l-2-3.4-2.3.6-2-.9L13 3Z"
          transform="translate(1)"
        />
        <circle cx="12" cy="12" r="3" />
      </svg>
    </Button>
  </Corner>
  <Popover id="language-settings" label={$t('settings')} placement="end">
    <Select
      label={$t('language')}
      value={$locale}
      options={languages.map((language) => ({
        value: language.id,
        label: language.label,
        lang: language.id === 'de-cool' ? 'de' : language.id,
      }))}
      onchange={setLanguage}
    />
  </Popover>
</div>

<style>
  /* Only fixed and top-layer children: take no space in the page layout. */
  .settings {
    display: contents;
  }
</style>
