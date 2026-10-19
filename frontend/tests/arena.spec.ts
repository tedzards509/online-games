import { test, expect, type Page } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import type { GameSnapshot, Snapshot } from '../src/lib/types';

function fixture(game: Partial<GameSnapshot> = {}): Snapshot {
  const members = [
    { id: 0, nickname: 'Alice' },
    { id: 1, nickname: 'Bob' },
  ];
  return {
    code: 'ABC234',
    members,
    host: 0,
    you: 0,
    revision: 3,
    config: { game: 'tic_tac_toe', rules: {} },
    seating: [0, 1],
    replays: [],
    game: {
      id: 'test-game',
      config: { game: 'tic_tac_toe', rules: {} },
      players: members,
      you: 0,
      can_finish: false,
      turn: { players: [0, 1], number: 0 },
      outcome: null,
      view: { game: 'tic_tac_toe', state: { board: Array(9).fill(null) } },
      legal_actions: [{ game: 'tic_tac_toe', action: { index: 0 } }],
      ...game,
    },
  };
}

async function mockMembership(page: Page) {
  await page.addInitScript(() =>
    localStorage.setItem(
      'arena:ABC234',
      JSON.stringify({ code: 'ABC234', member: 0, token: 'test-member' }),
    ),
  );
}

test('multiple eligible players become a single responder', async ({
  page,
}) => {
  const room = fixture();
  await mockMembership(page);
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'game') {
        expect(message.game_id).toBe('test-game');
        expect(message.revision).toBe(3);
        expect(message.message).toEqual({
          type: 'move',
          action: room.game!.legal_actions[0],
        });
        room.revision++;
        room.game!.turn = { players: [1], number: 1 };
        room.game!.view = {
          game: 'tic_tac_toe',
          state: { board: [0, ...Array(8).fill(null)] },
        };
        room.game!.legal_actions = [];
      }
      socket.send(
        JSON.stringify({
          type: 'snapshot',
          room,
          acknowledged: message.type === 'game',
        }),
      );
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

test('another member snapshot does not acknowledge a queued move', async ({
  page,
}) => {
  const room = fixture();
  await mockMembership(page);
  let moves = 0;
  let acknowledge: () => void = () => {
    throw new Error('No queued move');
  };
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'game') {
        moves++;
        room.revision = 4;
        acknowledge = () => {
          room.revision = 5;
          room.game!.legal_actions = [
            { game: 'tic_tac_toe', action: { index: 1 } },
          ];
          socket.send(
            JSON.stringify({ type: 'snapshot', room, acknowledged: true }),
          );
        };
      }
      socket.send(
        JSON.stringify({ type: 'snapshot', room, acknowledged: false }),
      );
    });
  });
  await page.goto('/room/ABC234');
  await page
    .getByRole('button', { name: 'Cell 1: empty', exact: true })
    .click();
  await expect.poll(() => moves).toBe(1);
  await expect(
    page.getByLabel('Tic-Tac-Toe board').locator('button:enabled'),
  ).toHaveCount(0);
  acknowledge();
  await expect(
    page.getByRole('button', { name: 'Cell 2: empty', exact: true }),
  ).toBeEnabled();
  expect(moves).toBe(1);
});

test('Crazy Eights: choose a suit, draw voluntarily, and resolve draw two', async ({
  page,
}) => {
  const room = fixture({
    config: {
      game: 'crazy_eights',
      rules: {
        ace_are_you_strong: true,
        jack_skip: false,
        ten_reverse: false,
        two_draw_two: true,
      },
    },
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
      ...(['clubs', 'diamonds', 'hearts', 'spades'] as const).map((suit) => ({
        game: 'crazy_eights' as const,
        action: { type: 'play' as const, index: 0, suit },
      })),
      { game: 'crazy_eights', action: { type: 'draw' } },
    ],
  });
  await mockMembership(page);
  const moves: unknown[] = [];
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'game') {
        moves.push(message.message.action);
        room.revision++;
        if (moves.length === 2) {
          room.game!.legal_actions = [
            { game: 'crazy_eights', action: { type: 'draw' } },
          ];
          if (room.game!.view?.game === 'crazy_eights')
            room.game!.view.state.pending_draw = 'two';
        }
      }
      socket.send(
        JSON.stringify({
          type: 'snapshot',
          room,
          acknowledged: message.type === 'game',
        }),
      );
    });
  });
  await page.goto('/room/ABC234');
  await expect(
    page
      .getByLabel('Your hand', { exact: true })
      .getByRole('button', { name: '8♥', exact: true }),
  ).toBeEnabled();
  await page.getByRole('button', { name: 'Draw', exact: true }).click();
  await expect.poll(() => moves.length).toBe(1);
  expect(moves[0]).toEqual({ game: 'crazy_eights', action: { type: 'draw' } });
  await page.getByRole('button', { name: '8♥', exact: true }).click();
  await expect(page.getByLabel('Choose suit')).toBeVisible();
  await page
    .getByRole('button', { name: 'Choose spades', exact: true })
    .click();
  await expect.poll(() => moves.length).toBe(2);
  expect(moves[1]).toEqual({
    game: 'crazy_eights',
    action: { type: 'play', index: 0, suit: 'spades' },
  });
  await expect(page.getByLabel('Choose suit')).toHaveCount(0);
  await page.getByRole('button', { name: 'Draw two', exact: true }).click();
  await expect.poll(() => moves.length).toBe(3);
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
async function create(page: Page) {
  await page.goto('/');
  await page.getByLabel('Nickname').fill('Alice');
  await page.getByRole('button', { name: 'Create room' }).click();
  await expect(
    page.getByRole('heading', { name: 'Room', exact: true }),
  ).toBeVisible();
  return new URL(page.url()).pathname;
}
async function join(page: Page, path: string, name: string) {
  await page.goto(path);
  await page.getByLabel('Nickname').fill(name);
  await page.getByRole('button', { name: 'Join room' }).click();
  await expect(page.getByLabel('Members')).toContainText(name);
}
async function seat(host: Page, name: string) {
  await host.getByRole('button', { name: `Seat ${name}`, exact: true }).click();
  await expect(host.getByLabel('Seating')).toContainText(name);
}

test('room settings update live; seating, bench, abort, and switching games', async ({
  browser,
}) => {
  const contexts = await Promise.all([0, 1, 2].map(() => browser.newContext()));
  const [host, guest, late] = await Promise.all(
    contexts.map((c) => c.newPage()),
  );
  const path = await create(host);
  expect((await snapshot(host)).config).toBeNull();
  await join(guest, path, 'Bob');
  await expect(host.getByLabel('Bench')).toContainText('Bob');
  await expect(
    guest.getByText('Waiting for host to choose a game'),
  ).toBeVisible();
  await host
    .getByRole('combobox', { name: 'Game', exact: true })
    .selectOption('crazy_eights');
  await host.getByRole('checkbox', { name: /2: Next/ }).check();
  await expect(guest.getByRole('region', { name: 'Game rules' })).toContainText(
    '2: Next player draws two: On',
  );
  await expect(
    guest.getByRole('combobox', { name: 'Game', exact: true }),
  ).toHaveCount(0);
  await seat(host, 'Bob');
  await seat(host, 'Alice');
  expect((await snapshot(host)).seating).toEqual([1, 0]);
  await host
    .getByRole('button', { name: 'Move Alice up', exact: true })
    .click();
  await expect.poll(async () => (await snapshot(host)).seating).toEqual([0, 1]);
  await host.getByRole('button', { name: 'Start game', exact: true }).click();
  await expect(
    host.getByRole('heading', { name: 'Crazy Eights', exact: true }),
  ).toBeVisible();
  await expect(
    host.getByRole('combobox', { name: 'Game', exact: true }),
  ).toBeDisabled();
  await expect(
    host.getByRole('button', { name: 'Bench Bob', exact: true }),
  ).toBeDisabled();
  await join(late, path, 'Carol');
  await expect(
    late.getByText('You are on the bench', { exact: true }),
  ).toBeVisible();
  expect((await snapshot(late)).game.view).toBeNull();
  await expect(
    host.getByRole('button', { name: 'Seat Carol', exact: true }),
  ).toBeDisabled();
  const saved = await snapshot(guest);
  await guest.reload();
  await expect(guest.getByLabel('Your hand', { exact: true })).toBeVisible();
  expect((await snapshot(guest)).game.you).toBe(saved.game.you);
  await host.getByRole('button', { name: 'Abort game', exact: true }).click();
  await expect(
    host.getByRole('heading', { name: 'Room', exact: true }),
  ).toBeVisible();
  expect((await snapshot(host)).replays).toEqual([]);
  await host
    .getByRole('combobox', { name: 'Game', exact: true })
    .selectOption('tic_tac_toe');
  await host.getByRole('button', { name: 'Bench Alice', exact: true }).click();
  await expect(
    host.getByRole('button', { name: 'Seat Alice', exact: true }),
  ).toBeEnabled();
  await seat(host, 'Carol');
  await host.getByRole('button', { name: 'Start game', exact: true }).click();
  await expect(
    host.getByText('You are on the bench', { exact: true }),
  ).toBeVisible();
  await expect(guest.getByText('Your turn', { exact: true })).toBeVisible();
  for (const [page, index] of [
    [guest, 1],
    [late, 4],
    [guest, 2],
    [late, 5],
    [guest, 3],
  ] as const) {
    await page
      .getByRole('button', { name: `Cell ${index}: empty`, exact: true })
      .click();
    await expect.poll(async () => (await snapshot(host)).game.view).toBeNull();
    await expect(
      page.getByRole('button', {
        name: `Cell ${index}: ${page === guest ? 'X' : 'O'}`,
        exact: true,
      }),
    ).toBeVisible();
  }
  await expect(host.getByText('Bob won', { exact: true })).toBeVisible();
  await host.getByText('Replays (1)', { exact: true }).click();
  const download = host.waitForEvent('download');
  await host.getByRole('button', { name: /Game 1 · Tic-Tac-Toe/ }).click();
  const replay = JSON.parse(
    await readFile((await (await download).path())!, 'utf8'),
  );
  expect(replay.game).toBe('tic_tac_toe');
  await host.getByRole('button', { name: 'Play again', exact: true }).click();
  await expect(
    guest.getByRole('button', { name: 'Cell 1: empty', exact: true }),
  ).toBeEnabled();
  for (const context of contexts) await context.close();
});

test('The Crew: benched host records results independently of player order', async ({
  browser,
}) => {
  const contexts = await Promise.all(
    [0, 1, 2, 3].map(() => browser.newContext()),
  );
  const pages = await Promise.all(contexts.map((c) => c.newPage()));
  const [host, ...players] = pages;
  const path = await create(host);
  for (const [i, page] of players.entries())
    await join(page, path, ['Bob', 'Carol', 'Dave'][i]);
  await host
    .getByRole('combobox', { name: 'Game', exact: true })
    .selectOption('the_crew');
  for (const name of ['Dave', 'Bob', 'Carol']) await seat(host, name);
  await host.getByRole('button', { name: 'Start game', exact: true }).click();
  await expect(
    host.getByText('You are on the bench', { exact: true }),
  ).toBeVisible();
  for (const page of players) {
    await expect(page.getByLabel('Your hand', { exact: true })).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Finish: won', exact: true }),
    ).toHaveCount(0);
  }
  await host.getByRole('button', { name: 'Finish: won', exact: true }).click();
  await expect(
    host.getByText('Dave, Bob, Carol won', { exact: true }),
  ).toBeVisible();
  await host.getByRole('button', { name: 'Play again', exact: true }).click();
  await host.getByRole('button', { name: 'Finish: lost', exact: true }).click();
  for (const page of pages)
    await expect(page.getByText('Lost', { exact: true })).toBeVisible();
  expect((await snapshot(host)).replays).toHaveLength(2);
  for (const context of contexts) await context.close();
});

test('Tempel des Schreckens: configured key, hidden cards, rounds, and reconnect', async ({
  browser,
}) => {
  const contexts = await Promise.all([0, 1, 2].map(() => browser.newContext()));
  const pages = await Promise.all(contexts.map((c) => c.newPage()));
  const [host, guest, third] = pages;
  const names = ['Carol', 'Alice', 'Bob'];
  const errors: string[] = [];
  for (const page of pages)
    page.on('pageerror', (error) => errors.push(error.message));
  const path = await create(host);
  await join(guest, path, 'Bob');
  await join(third, path, 'Carol');
  await host
    .getByRole('combobox', { name: 'Game', exact: true })
    .selectOption('tempel_des_schreckens');
  for (const name of names) await seat(host, name);
  await host
    .getByRole('combobox', { name: 'Initial key holder', exact: true })
    .selectOption({ label: 'Bob' });
  await expect(guest.getByRole('region', { name: 'Game rules' })).toContainText(
    'Initial key: Player 3',
  );
  await host.getByRole('button', { name: 'Start game', exact: true }).click();
  await expect(guest.getByText('Key: Bob', { exact: true })).toBeVisible();
  const role = (await snapshot(guest)).game.view.state.role;
  await guest.reload();
  await expect(guest.getByLabel('Your role')).toContainText(role);
  for (const page of pages) {
    await expect(
      page.getByRole('button', { name: / card \d+: hidden$/ }),
    ).toHaveCount(15);
    await expect(
      page
        .getByRole('button', { name: / card \d+: hidden$/ })
        .and(page.locator(':enabled')),
    ).toHaveCount(0);
    await page.getByRole('button', { name: 'Ready', exact: true }).click();
  }
  await guest
    .getByRole('button', { name: 'Alice card 1: hidden', exact: true })
    .click();
  await expect(host.getByText('Key: Alice', { exact: true })).toBeVisible();
  const playerPages = [third, host, guest];
  for (let step = 0; step < 30; step++) {
    const room = await snapshot(host);
    if (room.game.outcome) break;
    const page = playerPages[room.game.turn.players[0]];
    const personal = await snapshot(page);
    const action = personal.game.legal_actions[0].action;
    if (action.type === 'ready')
      await page.getByRole('button', { name: 'Ready', exact: true }).click();
    else
      await page
        .getByRole('button', {
          name: `${names[action.player]} card ${action.index + 1}: hidden`,
          exact: true,
        })
        .click();
    await expect
      .poll(async () => (await snapshot(host)).revision)
      .toBe(room.revision + 1);
  }
  expect((await snapshot(host)).game.outcome).not.toBeNull();
  await host.getByRole('button', { name: 'Play again', exact: true }).click();
  await expect(host.getByText('Key: Bob', { exact: true })).toBeVisible();
  expect(errors).toEqual([]);
  for (const context of contexts) await context.close();
});

for (const [kind, name, board] of [
  ['mancala', 'Mancala', 'Mancala board'],
  ['crazy_eights', 'Crazy Eights', 'Your hand'],
  ['shithead', 'Shithead', 'Your hand'],
  ['skyjo', 'Skyjo', "Alice's grid"],
  ['tic_tac_toe', 'Tic-Tac-Toe', 'Tic-Tac-Toe board'],
] as const) {
  test(`${name}: create, seat, start, and reconnect`, async ({ browser }) => {
    const contexts = await Promise.all([0, 1].map(() => browser.newContext()));
    const [host, guest] = await Promise.all(contexts.map((c) => c.newPage()));
    const path = await create(host);
    await join(guest, path, 'Bob');
    await host
      .getByRole('combobox', { name: 'Game', exact: true })
      .selectOption(kind);
    await seat(host, 'Bob');
    await seat(host, 'Alice');
    await host.getByRole('button', { name: 'Start game', exact: true }).click();
    await expect(
      host.getByRole('heading', { name, exact: true }),
    ).toBeVisible();
    await expect(host.getByLabel(board, { exact: true })).toBeVisible();
    const saved = await snapshot(host);
    await host.reload();
    await expect(host.getByLabel(board, { exact: true })).toBeVisible();
    expect((await snapshot(host)).game).toEqual(saved.game);
    for (const context of contexts) await context.close();
  });
}
