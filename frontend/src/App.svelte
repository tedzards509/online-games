<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from './lib/i18n';
  import {
    createRoom,
    downloadReplay,
    joinRoom,
    loadMembership,
    saveMembership,
  } from './lib/api';
  import { RoomConnection } from './lib/connection.svelte';
  import type { Membership } from './lib/types';
  import GameBoard from './components/GameBoard.svelte';
  import LandingPage from './components/LandingPage.svelte';
  import LanguageSettings from './components/LanguageSettings.svelte';
  import GameStatus from './components/room/GameStatus.svelte';
  import RoomLobby from './components/room/RoomLobby.svelte';
  import RoomMenu from './components/room/RoomMenu.svelte';
  import Notice from './components/ui/Notice.svelte';

  function pathCode(): string {
    return (
      /^\/room\/([A-Z2-9]{6})\/?$/i
        .exec(location.pathname)?.[1]
        .toUpperCase() ?? ''
    );
  }
  const initialCode = pathCode();
  let code = $state(initialCode);
  let member = $state<Membership | null>(
    initialCode ? loadMembership(initialCode) : null,
  );
  let connection = $state<RoomConnection | null>(null);
  let nickname = $state(localStorage.getItem('arena-nickname') ?? '');
  let joinCode = $state(initialCode);
  let busy = $state(false);
  let error = $state('');
  let entryError = $state<string | null>(null);
  const room = $derived(connection?.room);
  const game = $derived(room?.game);
  /** Boards that own the whole viewport and show their own turn status. */
  const immersive = $derived(game?.config.game === 'the_crew');
  const disabled = $derived(
    !!connection?.pending || connection?.status !== 'connected',
  );

  function navigate(next: string): void {
    history.pushState(null, '', next ? `/room/${next}` : '/');
    syncLocation();
  }
  function syncLocation(): void {
    code = pathCode();
    joinCode = code;
    member = code ? loadMembership(code) : null;
    error = '';
    entryError = null;
  }
  onMount(() => {
    window.addEventListener('popstate', syncLocation);
    return () => window.removeEventListener('popstate', syncLocation);
  });
  $effect(() => {
    const current = member;
    if (!current) {
      connection = null;
      return;
    }
    const active = new RoomConnection(current);
    connection = active;
    return () => active.close();
  });

  async function enter(create: boolean): Promise<void> {
    if (busy) return;
    busy = true;
    error = '';
    entryError = null;
    try {
      const target = joinCode.trim().toUpperCase();
      const existing = !create ? loadMembership(target) : null;
      const membership =
        existing ??
        (await (create ? createRoom(nickname) : joinRoom(target, nickname)));
      saveMembership(membership);
      localStorage.setItem('arena-nickname', nickname.trim());
      navigate(membership.code);
    } catch (cause) {
      // The HTTP API supplies English messages; translate expected entry errors.
      const messages: Record<string, string> = {
        'Room or replay not found': 'roomNotFound',
        'Nickname already taken': 'nicknameTaken',
        'Use a nickname of 1–24 characters': 'nicknameInvalid',
      };
      entryError =
        (cause instanceof Error && messages[cause.message]) || 'enterFailed';
    } finally {
      busy = false;
    }
  }

  async function download(id: string): Promise<void> {
    if (!member) return;
    try {
      await downloadReplay(member.code, id, member.token);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Download failed';
    }
  }
  function showError(message: string): void {
    error = message;
  }
</script>

<svelte:head
  ><title>{code ? `${code} · ` : ''}Friends' Arena</title></svelte:head
>

<main class:entry={!member} class:immersive lang={member ? 'en' : undefined}>
  <LanguageSettings />

  {#if entryError || error || connection?.error || (member && connection?.status !== 'connected')}
    <div class="notices">
      {#if entryError}<Notice tone="error">{$t(entryError)}</Notice>{/if}
      {#if error || connection?.error}<Notice tone="error"
          >{error || connection?.error}</Notice
        >{/if}
      {#if member && connection?.status !== 'connected'}
        <div class="connection-notice">
          <Notice>{room ? 'Reconnecting…' : 'Connecting…'}</Notice>
        </div>
      {/if}
    </div>
  {/if}

  {#if !member}
    <LandingPage bind:nickname bind:joinCode invited={!!code} {busy} {enter} />
  {:else if room && !game}
    <RoomLobby
      {room}
      connection={connection!}
      {disabled}
      {download}
      onerror={showError}
    />
  {:else if room && game}
    <RoomMenu
      {room}
      {game}
      connection={connection!}
      {disabled}
      withStatus={immersive}
      {download}
      onerror={showError}
    />
    {#if !immersive}<header><GameStatus {game} /></header>{/if}
    <GameBoard
      room={game}
      roomInfo={{ code: room.code, host: room.host }}
      {disabled}
      move={(action) => connection?.move(action)}
    />
  {/if}
</main>

<style>
  main {
    display: grid;
    align-content: start;
    gap: var(--arena-space-3);
    max-width: var(--arena-page-width);
    min-height: 100svh;
    margin: auto;
    /* Keeps content clear of the fixed corner buttons. */
    padding: var(--arena-space-3) 5.5rem var(--arena-space-4);
  }
  main.entry {
    align-content: center;
    max-width: none;
    padding-block: 6rem;
  }
  /* The board fills exactly one viewport and positions its own chrome. */
  main.immersive {
    position: fixed;
    inset: 0;
    max-width: none;
    height: 100vh;
    height: 100dvh;
    grid-template-rows: minmax(0, 1fr);
    gap: 0;
    padding: 0;
    overflow: hidden;
  }
  .notices {
    position: fixed;
    top: 0.75rem;
    left: 50%;
    z-index: 9;
    display: grid;
    gap: var(--arena-space-1);
    width: max-content;
    max-width: calc(100% - 10rem);
    transform: translateX(-50%);
  }
  @media (max-width: 40rem) {
    main {
      padding: var(--arena-space-2) var(--arena-space-2) 6rem;
    }
    main.entry {
      padding-block: var(--arena-space-4) 6rem;
    }
    .notices {
      max-width: calc(100% - 2rem);
    }
  }
</style>
