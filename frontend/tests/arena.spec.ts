import { test, expect, type Page } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import type { Snapshot } from '../src/lib/types';

test('multiple eligible seats become a single responder', async ({ page }) => {
  // Protocol fixture: existing Tic-Tac-Toe rules remain sequential.
  const room: Snapshot = {
    code: 'ABC234',
    game: 'tic_tac_toe',
    config: { game: 'tic_tac_toe', rules: {} },
    you: 0,
    revision: 3,
    players: [{ nickname: 'Alice' }, { nickname: 'Bob' }],
    turn: { players: [0, 1], number: 0 },
    outcome: null,
    view: { game: 'tic_tac_toe', state: { board: Array(9).fill(null) } },
    legal_actions: [{ game: 'tic_tac_toe', action: { index: 0 } }],
    replays: [],
  };
  await page.addInitScript(() =>
    localStorage.setItem(
      'arena:ABC234',
      JSON.stringify({
        code: 'ABC234',
        player: 0,
        token: 'test-seat',
      }),
    ),
  );
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'move') {
        expect(message.revision).toBe(3);
        expect(message.action).toEqual(room.legal_actions[0]);
        room.revision++;
        room.turn = { players: [1], number: 1 };
        room.view = {
          game: 'tic_tac_toe',
          state: { board: [0, ...Array(8).fill(null)] },
        };
        room.legal_actions = [];
      }
      socket.send(JSON.stringify({ type: 'snapshot', room }));
    });
  });
  await page.goto('/room/ABC234');
  await expect(page.getByText('Your turn', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Players').locator('li.active')).toHaveCount(2);
  await page
    .getByRole('button', { name: 'Cell 1: empty', exact: true })
    .click();
  await expect(page.getByText("Bob's turn", { exact: true })).toBeVisible();
  await expect(page.getByLabel('Players').locator('li.active')).toHaveCount(1);
  await expect(
    page.getByLabel('Tic-Tac-Toe board').locator('button:enabled'),
  ).toHaveCount(0);
});

async function snapshot(page: Page) {
  return page.evaluate(async () => {
    const code = location.pathname.split('/').pop();
    const member = JSON.parse(localStorage.getItem(`arena:${code}`)!);
    return (
      await fetch(`/api/rooms/${code}`, {
        headers: { Authorization: `Bearer ${member.token}` },
      })
    ).json();
  });
}

test('Crazy Eights: ordinary eights submit directly and penalties offer draw two', async ({
  page,
}) => {
  const room: Snapshot = {
    code: 'ABC234',
    game: 'crazy_eights',
    you: 0,
    revision: 3,
    config: {
      game: 'crazy_eights',
      rules: {
        ace_are_you_strong: true,
        jack_skip: false,
        ten_reverse: false,
        eight_wild: false,
        two_draw_two: true,
      },
    },
    players: [{ nickname: 'Alice' }, { nickname: 'Bob' }],
    turn: { players: [0], number: 0 },
    outcome: null,
    replays: [],
    view: {
      game: 'crazy_eights',
      state: {
        hand: [
          { rank: 8, suit: 'hearts' },
          { rank: 4, suit: 'clubs' },
        ],
        counts: [2, 2],
        top: { rank: 3, suit: 'hearts' },
        suit: 'hearts',
        stock: 10,
        reversed: false,
        pending_draw: null,
      },
    },
    legal_actions: [
      { game: 'crazy_eights', action: { type: 'play', index: 0, suit: null } },
    ],
  };
  await page.addInitScript(() =>
    localStorage.setItem(
      'arena:ABC234',
      JSON.stringify({ code: 'ABC234', player: 0, token: 'test-seat' }),
    ),
  );
  const moves: unknown[] = [];
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'move') {
        moves.push(message.action);
        room.revision++;
        room.legal_actions = [
          { game: 'crazy_eights', action: { type: 'draw' } },
        ];
        if (room.view?.game === 'crazy_eights')
          room.view.state.pending_draw = 'two';
      }
      socket.send(JSON.stringify({ type: 'snapshot', room }));
    });
  });
  await page.goto('/room/ABC234');
  await page
    .getByLabel('Your hand', { exact: true })
    .getByRole('button', { name: '8♥', exact: true })
    .click();
  await expect.poll(() => moves.length).toBe(1);
  expect(moves[0]).toEqual({
    game: 'crazy_eights',
    action: { type: 'play', index: 0, suit: null },
  });
  await expect(page.getByLabel('Choose suit')).toHaveCount(0);
  await page.getByRole('button', { name: 'Draw two', exact: true }).click();
  await expect.poll(() => moves.length).toBe(2);
  expect(moves[1]).toEqual({ game: 'crazy_eights', action: { type: 'draw' } });
});

test('Crazy Eights: host selects fixed rules before opening the room', async ({
  browser,
}) => {
  const hostContext = await browser.newContext();
  const guestContext = await browser.newContext();
  const host = await hostContext.newPage();
  const guest = await guestContext.newPage();
  await host.goto('/');
  await host.getByLabel('Nickname').fill('Alice');
  await host
    .getByRole('combobox', { name: 'Game', exact: true })
    .selectOption('crazy_eights');
  await expect(
    host.getByRole('checkbox', { name: '8: Choose suit' }),
  ).toBeChecked();
  for (const label of [
    'A: Are you strong',
    'J: Skip',
    '10: Reverse',
    '2: Next',
  ]) {
    await host.getByRole('checkbox', { name: new RegExp(label) }).check();
  }
  await host.getByRole('checkbox', { name: '8: Choose suit' }).uncheck();
  const creating = host.waitForRequest(
    (request) =>
      request.url().endsWith('/api/rooms') && request.method() === 'POST',
  );
  await host.getByRole('button', { name: 'Create room' }).click();
  const config = {
    game: 'crazy_eights',
    rules: {
      ace_are_you_strong: true,
      jack_skip: true,
      ten_reverse: true,
      eight_wild: false,
      two_draw_two: true,
    },
  };
  expect((await creating).postDataJSON().config).toEqual(config);
  await expect(
    host.getByRole('heading', { name: 'Crazy Eights', exact: true }),
  ).toBeVisible();
  await guest.goto(new URL(host.url()).pathname);
  await guest.getByLabel('Nickname').fill('Bob');
  await guest.getByRole('button', { name: 'Join room' }).click();
  for (const page of [host, guest]) {
    await expect(
      page.getByRole('region', { name: 'Game rules' }),
    ).toContainText('8: Choose suit: Off');
    await expect(
      page.getByRole('region', { name: 'Game rules' }),
    ).toContainText('2: Next player draws two: On');
    await expect(page.getByRole('checkbox')).toHaveCount(0);
    expect((await snapshot(page)).config).toEqual(config);
  }
  await host.getByRole('button', { name: 'Start game' }).click();
  await expect(host.getByText('Your turn', { exact: true })).toBeVisible();
  for (const page of [host, guest]) {
    await page.reload();
    await expect(
      page.getByRole('region', { name: 'Game rules' }),
    ).toContainText('8: Choose suit: Off');
    expect((await snapshot(page)).config).toEqual(config);
  }
  await hostContext.close();
  await guestContext.close();
});

for (const game of [
  'Mancala',
  'Crazy Eights',
  'Shithead',
  'Skyjo',
  'Tic-Tac-Toe',
]) {
  test(`${game}: create, join, move, and reconnect`, async ({ browser }) => {
    const hostContext = await browser.newContext();
    const guestContext = await browser.newContext();
    const host = await hostContext.newPage();
    const guest = await guestContext.newPage();
    const errors: string[] = [];
    host.on('pageerror', (error) => errors.push(error.message));
    guest.on('pageerror', (error) => errors.push(error.message));
    const received: number[] = [];
    guest.on('websocket', (socket) =>
      socket.on('framereceived', ({ payload }) => {
        const message = JSON.parse(String(payload));
        if (message.type === 'snapshot') received.push(message.room.revision);
      }),
    );

    await host.goto('/');
    await host.getByLabel('Nickname').fill('Alice');
    await host
      .getByRole('combobox', { name: 'Game', exact: true })
      .selectOption({ label: game });
    await host.getByRole('button', { name: 'Create room' }).click();
    await expect(
      host.getByRole('heading', { name: game, exact: true }),
    ).toBeVisible();
    const path = new URL(host.url()).pathname;
    await guest.goto(path);
    await guest.getByLabel('Nickname').fill('Bob');
    await guest.getByRole('button', { name: 'Join room' }).click();
    await expect(host.getByLabel('Players')).toContainText('Bob');
    await expect(guest.getByLabel('Players')).toContainText('Alice');
    await host.getByRole('button', { name: 'Start game' }).click();
    await expect(host.getByText('Your turn', { exact: true })).toBeVisible();
    await expect(
      guest.getByText("Alice's turn", { exact: true }),
    ).toBeVisible();
    const before = (await snapshot(host)).revision;

    if (game === 'Mancala') {
      await host
        .getByRole('button', { name: 'Pit 1: 4 stones', exact: true })
        .click();
      await expect(guest.getByText('Your turn', { exact: true })).toBeVisible();
    } else if (game === 'Tic-Tac-Toe') {
      for (const [page, cell] of [
        [host, 1],
        [guest, 4],
        [host, 2],
        [guest, 5],
        [host, 3],
      ] as const) {
        await page
          .getByRole('button', { name: `Cell ${cell}: empty`, exact: true })
          .click();
      }
      for (const page of [host, guest]) {
        await expect(
          page.getByText('Alice won', { exact: true }),
        ).toBeVisible();
        await expect(
          page.getByLabel('Tic-Tac-Toe board').locator('button:enabled'),
        ).toHaveCount(0);
        await expect(
          page.getByRole('button', { name: 'Cell 3: X', exact: true }),
        ).toBeVisible();
      }
      await host.getByText('Replays (1)', { exact: true }).click();
      const downloading = host.waitForEvent('download');
      await host.getByRole('button', { name: 'Game 1 ↓', exact: true }).click();
      const download = await downloading;
      const replay = JSON.parse(
        await readFile((await download.path())!, 'utf8'),
      );
      expect(replay.game).toBe('tic_tac_toe');
      expect(replay.records).toHaveLength(5);
      expect(replay.records[4].events.at(-1)).toEqual({
        type: 'finished',
        outcome: { winners: [0], scores: null },
      });
      await host
        .getByRole('button', { name: 'Play again', exact: true })
        .click();
      await expect(host.getByText('Your turn', { exact: true })).toBeVisible();
      await expect(
        host.getByRole('button', { name: /^Cell \d: empty$/ }),
      ).toHaveCount(9);
    } else if (game === 'Skyjo') {
      const grid = host.getByRole('region', { name: "Alice's grid" });
      // A labelled section is a region; each setup reveal is a separate command.
      await grid
        .getByRole('button', { name: 'Card 1: hidden', exact: true })
        .click();
      await expect(
        grid.getByRole('button', { name: /^Card 1: -?\d+$/ }),
      ).toBeVisible();
      await grid
        .getByRole('button', { name: 'Card 2: hidden', exact: true })
        .click();
      await expect(guest.getByText('Your turn', { exact: true })).toBeVisible();
    } else if (game === 'Shithead') {
      await host
        .getByLabel('Your hand', { exact: true })
        .locator('button:enabled')
        .first()
        .click();
      await host.getByRole('button', { name: 'Play', exact: true }).click();
    } else {
      const cards = host
        .getByLabel('Your hand', { exact: true })
        .locator('button:enabled');
      if (await cards.count()) {
        const card = cards.first();
        const wild = (await card.textContent())?.startsWith('8');
        await card.click();
        if (wild)
          await host
            .getByRole('button', { name: 'Choose clubs', exact: true })
            .click();
      } else await host.getByRole('button', { name: /Draw/ }).click();
      await expect(guest.getByText('Your turn', { exact: true })).toBeVisible();
    }

    await expect.poll(() => received.at(-1) ?? 0).toBeGreaterThan(before);
    const accepted = await snapshot(host);
    await host.reload();
    await expect(
      host.getByRole('heading', { name: game, exact: true }),
    ).toBeVisible();
    await expect
      .poll(async () => (await snapshot(host)).revision)
      .toBe(accepted.revision);
    expect(errors).toEqual([]);
    await hostContext.close();
    await guestContext.close();
  });
}

test('narrow viewport has no horizontal overflow', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 740 });
  await page.goto('/');
  await page.getByLabel('Nickname').fill('Alice');
  await page
    .getByRole('combobox', { name: 'Game', exact: true })
    .selectOption({ label: 'Mancala' });
  await page.getByRole('button', { name: 'Create room' }).click();
  await expect(
    page.getByRole('heading', { name: 'Mancala', exact: true }),
  ).toBeVisible();
  const code = new URL(page.url()).pathname.split('/').pop();
  await page.request.post(`/api/rooms/${code}/join`, {
    data: { nickname: 'Bob' },
  });
  await page.getByRole('button', { name: 'Start game' }).click();
  await expect(page.getByLabel('Mancala board')).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});
