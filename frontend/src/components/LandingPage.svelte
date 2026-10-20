<script lang="ts">
  import { t } from '../lib/i18n';
  import Button from './ui/Button.svelte';
  import TextField from './ui/TextField.svelte';
  import Panel from './ui/Panel.svelte';

  let {
    nickname = $bindable(''),
    joinCode = $bindable(''),
    invited = false,
    busy,
    enter,
  }: {
    nickname?: string;
    joinCode?: string;
    invited?: boolean;
    busy: boolean;
    enter: (create: boolean) => void;
  } = $props();
</script>

<section class="landing">
  <TextField
    label={$t('nickname')}
    bind:value={nickname}
    maxlength={24}
    autocomplete="nickname"
    required
  />
  <div class="choices" class:invited>
    {#if !invited}
      <Panel>
        <form
          onsubmit={(event) => {
            event.preventDefault();
            enter(true);
          }}
        >
          <Button type="submit" disabled={busy || !nickname.trim()}
            >{$t('createRoom')}</Button
          >
        </form>
      </Panel>
    {/if}
    <Panel>
      <form
        onsubmit={(event) => {
          event.preventDefault();
          enter(false);
        }}
      >
        <TextField
          label={$t('roomCode')}
          bind:value={joinCode}
          maxlength={6}
          minlength={6}
          pattern={'[A-Za-z2-9]{6}'}
          autocapitalize="characters"
          spellcheck={false}
          required
        />
        <Button
          variant="secondary"
          type="submit"
          disabled={busy ||
            !nickname.trim() ||
            !/^[A-Z2-9]{6}$/i.test(joinCode.trim())}>{$t('joinRoom')}</Button
        >
      </form>
    </Panel>
  </div>
</section>

<style>
  .landing {
    width: min(100%, var(--arena-entry-width));
    margin: auto;
    display: grid;
    gap: var(--arena-space-3);
    font-family: var(--arena-font);
    color: var(--arena-text);
  }
  .choices {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--arena-space-3);
  }
  .choices.invited {
    grid-template-columns: minmax(0, 1fr);
  }
  form {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: end;
    gap: var(--arena-space-2);
    margin: 0;
  }
  @media (max-width: 40rem) {
    .choices {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
