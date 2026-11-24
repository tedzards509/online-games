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
  const seat = page.getByLabel('Your seat').locator('[data-active]');
  const opponents = page.getByLabel('Players').locator('[data-active="true"]');
  await expect(seat).toHaveAttribute('data-active', 'true');
  await expect(opponents).toHaveCount(1);
  await page
    .getByRole('button', { name: 'Cell 1: empty', exact: true })
    .click();
  await expect(page.getByText("Bob's turn", { exact: true })).toBeVisible();
  await expect(seat).toHaveAttribute('data-active', 'false');
  await expect(opponents).toHaveCount(1);
  await expect(
    page.getByLabel('Tic-Tac-Toe board').locator('button:enabled'),
  ).toHaveCount(0);
});

test('a pressed control still clicks when released at its resting edge', async ({
  page,
}) => {
  const moves: unknown[] = [];
  await mockMembership(page);
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'game') moves.push(message.message);
      else socket.send(JSON.stringify({ type: 'snapshot', room: fixture() }));
    });
  });
  await page.goto('/room/ABC234');
  const cell = page.getByRole('button', { name: 'Cell 1: empty', exact: true });
  const box = (await cell.boundingBox())!;
  // The top edge strip that the pressed shift moves the cell away from.
  await page.mouse.move(box.x + box.width / 2, box.y + 2);
  await page.mouse.down();
  await page.waitForTimeout(200);
  await page.mouse.up();
  await expect.poll(() => moves.length).toBe(1);
});

test('the end screen shows results and gives only the host the next step', async ({
  page,
}) => {
  const room = fixture({
    outcome: { winners: [0], scores: [3, 1] },
    turn: { players: [], number: 5 },
    legal_actions: [],
  });
  const commands: { type: string }[] = [];
  await mockMembership(page);
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'authenticate')
        socket.send(JSON.stringify({ type: 'snapshot', room }));
      else commands.push(message);
    });
  });
  await page.goto('/room/ABC234');
  const results = page.getByRole('dialog', { name: 'Alice won' });
  await expect(results.getByRole('listitem')).toHaveText([
    /Alice · you\s*Winner\s*Score: 3/,
    /Bob\s*Score: 1/,
  ]);
  await results
    .getByRole('button', { name: 'View board', exact: true })
    .click();
  await expect(results).toBeHidden();
  await page
    .getByRole('button', { name: 'Room controls', exact: true })
    .click();
  await page.getByRole('button', { name: 'Results', exact: true }).click();
  await results
    .getByRole('button', { name: 'Play again', exact: true })
    .click();
  await expect.poll(() => commands.at(-1)?.type).toBe('start');
});

test('guests see the end screen without host commands', async ({ page }) => {
  const room = fixture({ outcome: { winners: [], scores: null }, you: 1 });
  room.you = 1;
  await page.addInitScript(() =>
    localStorage.setItem(
      'arena:ABC234',
      JSON.stringify({ code: 'ABC234', member: 1, token: 'guest' }),
    ),
  );
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage(() =>
      socket.send(JSON.stringify({ type: 'snapshot', room })),
    );
  });
  await page.goto('/room/ABC234');
  const results = page.getByRole('dialog', { name: 'Draw' });
  await expect(results.getByText('Waiting for the host')).toBeVisible();
  await expect(results.getByRole('button', { name: 'Play again' })).toHaveCount(
    0,
  );
});

test('Tempel shows the finished round until skipped', async ({ page }) => {
  const counts = { gold: 0, traps: 0, empty: 0 };
  const view = (round: number, cards: (string | null)[][]) => ({
    game: 'tempel_des_schreckens',
    state: {
      role: 'adventurer',
      hand_counts: { gold: 1, traps: 0, empty: 4 },
      cards,
      ready: [false, false, false],
      phase: round === 1 ? 'reveal' : 'inspect',
      key: round === 1 ? 2 : 1,
      round,
      round_reveals: round === 1 ? 2 : 0,
      revealed:
        round === 1
          ? { ...counts, empty: 2 }
          : { ...counts, empty: 2, traps: 1 },
      distribution: {
        adventurers: 2,
        guardians: 1,
        cards: { gold: 5, traps: 2, empty: 8 },
      },
    },
  });
  const room = (round: number, cards: (string | null)[][]) => {
    const base = fixture();
    const members = [...base.members, { id: 2, nickname: 'Carol' }];
    const config = {
      game: 'tempel_des_schreckens',
      rules: { initial_key: 0 },
    } as const;
    return {
      ...base,
      members,
      config,
      seating: [0, 1, 2],
      game: {
        ...base.game!,
        config,
        players: members,
        turn: { players: [round === 1 ? 2 : 0], number: 0 },
        view: view(round, cards),
        legal_actions: [],
      },
    } as Snapshot;
  };
  const hidden = Array(5).fill(null);
  let send = (_: Snapshot) => {};
  await mockMembership(page);
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    send = (snapshot) =>
      socket.send(JSON.stringify({ type: 'snapshot', room: snapshot }));
    socket.onMessage(() =>
      send(
        room(1, [
          ['empty', ...hidden.slice(1)],
          hidden,
          ['empty', ...hidden.slice(1)],
        ]),
      ),
    );
  });
  await page.goto('/room/ABC234');
  await expect(page.getByText('Round 1 / 4 · 2 / 3 revealed')).toBeVisible();
  send(room(2, [hidden, hidden, hidden]));
  await expect(
    page.getByText(/^Round 1 opened · next round in \d s$/),
  ).toBeVisible();
  // The final card is inferred from the revealed counts and the new key holder.
  await expect(
    page
      .getByRole('group', { name: 'Opened from Bob', exact: true })
      .getByRole('img', { name: 'trap', exact: true }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Skip', exact: true }).click();
  await expect(page.getByText('Round 2 / 4 · 0 / 3 revealed')).toBeVisible();
});

test('room and board copy follow the selected language', async ({ page }) => {
  await mockMembership(page);
  await page.addInitScript(() => localStorage.setItem('arena-language', 'de'));
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    socket.onMessage(() =>
      socket.send(JSON.stringify({ type: 'snapshot', room: fixture() })),
    );
  });
  await page.goto('/room/ABC234');
  await expect(page.getByText('Du bist dran', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Spieler')).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Feld 1: leer', exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByRole('button', { name: 'Raumsteuerung', exact: true }),
  ).toBeVisible();
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
      .getByRole('button', { name: '8 of hearts', exact: true }),
  ).toBeEnabled();
  await page.getByRole('button', { name: 'Draw', exact: true }).click();
  await expect.poll(() => moves.length).toBe(1);
  expect(moves[0]).toEqual({ game: 'crazy_eights', action: { type: 'draw' } });
  await page.getByRole('button', { name: '8 of hearts', exact: true }).click();
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
/** Moves a member to the end of the seating, so call order sets player order. */
async function seat(host: Page, name: string) {
  await expect(host.getByLabel('Members')).toContainText(name);
  const bench = host.getByRole('button', {
    name: `Bench ${name}`,
    exact: true,
  });
  if (await bench.count()) await bench.click();
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
  // Before any game exists, joining members take a seat.
  await expect(host.getByLabel('Seating')).toContainText('Bob');
  await expect(
    guest.getByText('Waiting for host to choose a game'),
  ).toBeVisible();
  await host.getByRole('radio', { name: 'Crazy Eights', exact: true }).check();
  await host.getByRole('checkbox', { name: /2: Next/ }).check();
  await expect(guest.getByRole('region', { name: 'Game rules' })).toContainText(
    '2: Next player draws two: On',
  );
  await expect(guest.getByRole('radio')).toHaveCount(0);
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
  await expect(host.getByRole('radio')).toHaveCount(0);
  await expect(
    host.getByRole('button', { name: 'Bench Bob', exact: true }),
  ).toHaveCount(0);
  await join(late, path, 'Carol');
  await expect(
    late.getByText('You are on the bench', { exact: true }),
  ).toBeVisible();
  expect((await snapshot(late)).game.view).toBeNull();
  await expect(
    host.getByRole('button', { name: 'Seat Carol', exact: true }),
  ).toHaveCount(0);
  const saved = await snapshot(guest);
  await guest.reload();
  await expect(guest.getByLabel('Your hand', { exact: true })).toBeVisible();
  expect((await snapshot(guest)).game.you).toBe(saved.game.you);
  await host
    .getByRole('button', { name: 'Room controls', exact: true })
    .click();
  await host.getByRole('button', { name: 'Abort game', exact: true }).click();
  await expect(
    host.getByRole('heading', { name: 'Room', exact: true }),
  ).toBeVisible();
  expect((await snapshot(host)).replays).toEqual([]);
  await host.getByRole('radio', { name: 'Tic-Tac-Toe', exact: true }).check();
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
  await expect(
    host.getByRole('heading', { name: 'Bob won', exact: true }),
  ).toBeVisible();
  await host.getByRole('button', { name: 'View board', exact: true }).click();
  await host
    .getByRole('button', { name: 'Room controls', exact: true })
    .click();
  await host.getByText('Replays (1)', { exact: true }).click();
  const download = host.waitForEvent('download');
  await host.getByRole('button', { name: /Game 1 · Tic-Tac-Toe/ }).click();
  const replay = JSON.parse(
    await readFile((await (await download).path())!, 'utf8'),
  );
  expect(replay.game).toBe('tic_tac_toe');
  await host.getByRole('button', { name: 'Results', exact: true }).click();
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
  await host.getByRole('radio', { name: 'The Crew', exact: true }).check();
  for (const name of ['Dave', 'Bob', 'Carol']) await seat(host, name);
  await host.getByRole('button', { name: 'Bench Alice', exact: true }).click();
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
  await host
    .getByRole('button', { name: 'Room controls', exact: true })
    .click();
  await host.getByRole('button', { name: 'Finish: won', exact: true }).click();
  await expect(
    host.getByRole('heading', { name: 'Dave, Bob, Carol won', exact: true }),
  ).toBeVisible();
  await host.getByRole('button', { name: 'Play again', exact: true }).click();
  await host
    .getByRole('button', { name: 'Room controls', exact: true })
    .click();
  await host.getByRole('button', { name: 'Finish: lost', exact: true }).click();
  for (const page of pages)
    await expect(
      page.getByRole('heading', { name: 'Lost', exact: true }),
    ).toBeVisible();
  await host.getByRole('button', { name: 'View board', exact: true }).click();
  await expect(
    host.locator('[role="status"]:visible').filter({ hasText: /^Lost$/ }),
  ).toBeVisible();
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
    .getByRole('radio', { name: 'Tempel des Schreckens', exact: true })
    .check();
  for (const name of names) await seat(host, name);
  await host
    .getByRole('combobox', { name: 'Initial key holder', exact: true })
    .selectOption({ label: 'Bob' });
  await expect(guest.getByRole('region', { name: 'Game rules' })).toContainText(
    'Initial key holder: Player 3',
  );
  await host.getByRole('button', { name: 'Start game', exact: true }).click();
  await expect(guest.getByText('Key: Bob', { exact: true })).toBeVisible();
  const role = (await snapshot(guest)).game.view.state.role;
  await guest.reload();
  await expect(guest.getByLabel('Your role')).toContainText(role);
  for (const page of pages) {
    await expect(
      page.getByRole('button', { name: / card \d+: hidden$/ }),
    ).toHaveCount(10);
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
    await host.getByRole('radio', { name, exact: true }).check();
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
