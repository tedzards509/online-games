import { test, expect, type Page } from '@playwright/test';
import type { Action, Schemas, Snapshot } from '../src/lib/types';

function fixture(count = 5, you = 0): Snapshot {
  const players = Array.from({ length: count }, (_, id) => ({
    id: id + 2,
    nickname: ['Alice', 'Bob', 'Carol', 'Dave', 'Erin'][id],
  }));
  const hand: Schemas['CrewCard'][] = [
    { color: 'blue', number: 1 },
    { color: 'blue', number: 6 },
    { color: 'green', number: 4 },
    { color: 'pink', number: 7 },
    { color: 'yellow', number: 9 },
    { color: 'black', number: 4 },
  ];
  const state: Schemas['CrewView'] = {
    hand,
    counts: Array(count).fill(8),
    commander: 0,
    communications: Array(count).fill(null),
    communication_played: Array(count).fill(false),
    tricks_won: Array(count).fill(0),
    trick: [],
    last_trick: null,
    next_player: you,
    awaiting_result: false,
  };
  state.counts[you] = hand.length;
  return {
    code: 'ABC234',
    members: [
      { id: 0, nickname: 'Host' },
      { id: 1, nickname: 'Bench' },
      ...players,
    ],
    host: players[1].id,
    you: players[you].id,
    revision: 3,
    seating: players.map((member) => member.id),
    config: { game: 'the_crew', rules: {} },
    replays: [],
    game: {
      id: 'crew-table',
      players,
      you,
      config: { game: 'the_crew', rules: {} },
      can_finish: false,
      outcome: null,
      turn: {
        number: 0,
        players: Array.from({ length: count }, (_, index) => index),
      },
      view: { game: 'the_crew', state },
      legal_actions: [
        { game: 'the_crew', action: { type: 'play', index: 0 } },
        { game: 'the_crew', action: { type: 'play', index: 1 } },
        {
          game: 'the_crew',
          action: { type: 'communicate', index: 2, declaration: 'only' },
        },
      ],
    },
  };
}

function view(room: Snapshot): Schemas['CrewView'] {
  if (room.game?.view?.game !== 'the_crew')
    throw new Error('Expected Crew fixture');
  return room.game.view.state;
}

async function mock(page: Page, room: Snapshot) {
  await page.addInitScript(
    (membership) =>
      localStorage.setItem('arena:ABC234', JSON.stringify(membership)),
    { code: 'ABC234', member: room.you, token: 'crew-member' },
  );
  const moves: Action[] = [];
  let publish: (acknowledged?: boolean) => void = () => {};
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    publish = (acknowledged = false) =>
      socket.send(JSON.stringify({ type: 'snapshot', room, acknowledged }));
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type === 'game') {
        moves.push(message.message.action);
        return;
      }
      publish();
    });
  });
  await page.goto('/room/ABC234');
  await expect(
    page.getByRole('button', { name: 'Play card', exact: true }),
  ).toBeVisible();
  return { moves, publish: (acknowledged = false) => publish(acknowledged) };
}

test('Crew selects before playing, fades illegal plays, and serializes pending commands', async ({
  page,
}) => {
  const room = fixture();
  const { moves, publish } = await mock(page, room);
  const play = page.getByRole('button', { name: 'Play card', exact: true });
  await expect(play).toBeDisabled();
  const illegal = page.getByRole('button', {
    name: 'Select Green 4',
    exact: true,
  });
  await illegal.click();
  await expect(illegal).toHaveAttribute('aria-pressed', 'true');
  await expect(illegal.locator('..')).toHaveCSS('opacity', '0.58');
  await expect(play).toBeDisabled();
  const blue = page.getByRole('button', { name: 'Select Blue 6', exact: true });
  await blue.click();
  expect(moves).toHaveLength(0);
  await expect(blue).toHaveAttribute('aria-pressed', 'true');
  await play.click();
  await expect
    .poll(() => moves)
    .toEqual([{ game: 'the_crew', action: { type: 'play', index: 1 } }]);
  await expect(blue).toBeDisabled();
  room.revision++;
  publish();
  await expect(blue).toBeDisabled();
  publish(true);
  await expect(blue).toBeEnabled();
  await expect(play).toBeDisabled();
});

test('Crew communication allows only server-provided declarations for the selected card', async ({
  page,
}) => {
  const room = fixture();
  const { moves } = await mock(page, room);
  await page
    .getByRole('button', { name: 'Select Green 4', exact: true })
    .click();
  await page.getByText('Communicate a card', { exact: true }).click();
  await expect(
    page.getByRole('button', { name: 'Highest', exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole('button', { name: 'Lowest', exact: true }),
  ).toBeDisabled();
  await page.getByRole('button', { name: 'Only', exact: true }).click();
  await expect
    .poll(() => moves)
    .toEqual([
      {
        game: 'the_crew',
        action: { type: 'communicate', index: 2, declaration: 'only' },
      },
    ]);
});

test('Crew card faces use the specified colors and redundant SVG shapes', async ({
  page,
}) => {
  await mock(page, fixture());
  for (const [name, color, shape] of [
    ['Blue 1', '#008bcf', 'circle'],
    ['Green 4', '#8cc63e', 'path'],
    ['Pink 7', '#e77cb1', 'rect'],
    ['Yellow 9', '#dfae41', 'path'],
  ]) {
    const face = page.getByRole('button', {
      name: `Select ${name}`,
      exact: true,
    });
    await expect(face.locator('svg').first()).toHaveAttribute('fill', color);
    await expect(face.locator(`svg ${shape}`).first()).toBeAttached();
  }
});

test('Crew never reuses a stale selection after a hand update or a rematch', async ({
  page,
}) => {
  const room = fixture();
  const { publish, moves } = await mock(page, room);
  await page
    .getByRole('button', { name: 'Select Blue 6', exact: true })
    .click();
  view(room).hand.splice(1, 1);
  room.revision++;
  publish();
  await expect(
    page.getByRole('button', { name: 'Play card', exact: true }),
  ).toBeDisabled();
  await page
    .getByRole('button', { name: 'Select Blue 1', exact: true })
    .click();
  room.game!.id = 'another-crew-game';
  room.revision++;
  publish();
  await expect(
    page.getByRole('button', { name: 'Select Blue 1', exact: true }),
  ).toHaveAttribute('aria-pressed', 'false');
  expect(moves).toHaveLength(0);
});

test('Crew markers show unused, communicated, played, commander and trick counts after reconnect', async ({
  page,
}) => {
  const room = fixture();
  const state = view(room);
  state.communications[1] = {
    card: { color: 'yellow', number: 8 },
    declaration: 'highest',
  };
  state.communications[2] = {
    card: { color: 'pink', number: 2 },
    declaration: 'only',
  };
  state.communications[3] = {
    card: { color: 'blue', number: 3 },
    declaration: 'lowest',
  };
  state.communication_played[3] = true;
  state.tricks_won[1] = 3;
  await mock(page, room);
  const bob = page.getByRole('group', {
    name: 'Communication · Bob',
    exact: true,
  });
  await expect(
    bob.getByRole('img', { name: 'Yellow 8', exact: true }),
  ).toBeVisible();
  await expect(bob.locator('.marker')).toHaveAttribute(
    'data-position',
    'highest',
  );
  const carol = page.getByRole('group', {
    name: 'Communication · Carol',
    exact: true,
  });
  await expect(carol.locator('.marker')).toHaveAttribute(
    'data-position',
    'only',
  );
  const dave = page.getByRole('group', {
    name: 'Communication · Dave',
    exact: true,
  });
  await expect(dave).toHaveAttribute('data-state', 'played');
  await expect(
    dave.getByRole('img', { name: 'Blue 3', exact: true }),
  ).toHaveCount(0);
  await expect(dave.locator('.marker')).toHaveCSS(
    'background-color',
    'rgb(182, 64, 50)',
  );
  const erin = page.getByRole('group', {
    name: 'Communication · Erin',
    exact: true,
  });
  await expect(erin).toHaveAttribute('data-state', 'ready');
  await expect(
    page.getByRole('img', { name: 'Bob · 3 tricks won', exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole('img', {
      name: 'Commander · original black-four holder',
      exact: true,
    }),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole('group', { name: 'Communication · Dave', exact: true }),
  ).toHaveAttribute('data-state', 'played');
});

test('Crew info uses member identity rather than seat for room role and shows public data only', async ({
  page,
}) => {
  const room = fixture(5, 2);
  const state = view(room);
  state.communications[1] = {
    card: { color: 'yellow', number: 8 },
    declaration: 'highest',
  };
  state.communication_played[1] = true;
  state.tricks_won[1] = 4;
  await mock(page, room);
  await page
    .getByRole('button', { name: 'Info about Bob', exact: true })
    .click();
  const info = page.getByRole('region', {
    name: 'Info about Bob',
    exact: true,
  });
  await expect(info).toContainText('ABC234');
  await expect(info.locator('dd').nth(1)).toHaveText('3');
  await expect(info).toContainText('Host');
  await expect(info).toContainText('Cards remaining');
  await expect(info).toContainText(
    'Yellow 8 · Highest · Communicated card played',
  );
  await page.keyboard.press('Escape');
  await expect(info).toBeHidden();
  const opponent = page.getByRole('region', { name: 'Bob', exact: true });
  await expect(opponent.locator('.compact .playing-card')).toHaveCount(8);
  await expect(opponent).not.toContainText('Blue 6');
});

test('Crew trick scatter is stable; labels and last trick remain available', async ({
  page,
}) => {
  const room = fixture();
  const state = view(room);
  state.trick = [
    { player: 3, card: { color: 'blue', number: 7 } },
    { player: 4, card: { color: 'blue', number: 8 } },
  ];
  state.next_player = 0;
  const { publish } = await mock(page, room);
  const trick = page.getByRole('region', {
    name: 'Current trick',
    exact: true,
  });
  await expect(trick).toContainText('Dave');
  await expect(trick).toContainText('Erin');
  const transforms = await trick
    .locator('figure')
    .evaluateAll((elements) =>
      elements.map((element) => getComputedStyle(element).transform),
    );
  room.revision++;
  publish();
  expect(
    await trick
      .locator('figure')
      .evaluateAll((elements) =>
        elements.map((element) => getComputedStyle(element).transform),
      ),
  ).toEqual(transforms);
  state.last_trick = { cards: [...state.trick], winner: 4 };
  state.trick = [];
  state.tricks_won[4] = 1;
  room.revision++;
  publish();
  await expect(
    page.getByRole('region', { name: 'Last trick', exact: true }),
  ).toContainText('Erin won the trick');
});

for (const [width, height, count] of [
  [1280, 900, 5],
  [768, 1024, 4],
  [390, 844, 5],
  [320, 568, 3],
] as const) {
  test(`Crew ${count} players at ${width}px keeps fans, controls, and info usable`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height });
    const room = fixture(count, 1);
    if (count === 3) {
      const state = view(room);
      state.hand = Array.from({ length: 14 }, (_, index) => ({
        color: index < 9 ? 'blue' : 'green',
        number: (index % 9) + 1,
      }));
      state.counts[1] = 14;
      room.game!.legal_actions = [
        { game: 'the_crew', action: { type: 'play', index: 13 } },
      ];
    }
    await mock(page, room);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
    ).toBe(width);
    const hand = page.getByRole('region', { name: 'Your hand', exact: true });
    const fan = hand.locator('.fan');
    await expect(fan.locator('button')).toHaveCount(view(room).hand.length);
    await fan.locator('button').last().focus();
    await page.keyboard.press('Space');
    await expect(fan.locator('button').last()).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    const infoButton = page.getByRole('button', {
      name: 'Info about Alice',
      exact: true,
    });
    expect((await infoButton.boundingBox())!.height).toBeGreaterThanOrEqual(44);
    await infoButton.click();
    await expect(
      page.getByRole('region', { name: 'Info about Alice', exact: true }),
    ).toBeVisible();
    await page.keyboard.press('Escape');
    for (const [player, member] of room.game!.players.entries()) {
      if (player === room.game!.you) continue;
      await page
        .getByRole('button', {
          name: `Info about ${member.nickname}`,
          exact: true,
        })
        .click();
      await expect(
        page.getByRole('region', {
          name: `Info about ${member.nickname}`,
          exact: true,
        }),
      ).toBeVisible();
      await page.keyboard.press('Escape');
    }
    for (const opponent of await page.locator('.opponent').all()) {
      const box = (await opponent.boundingBox())!;
      expect(box.x).toBeGreaterThanOrEqual(0);
      expect(box.x + box.width).toBeLessThanOrEqual(width);
    }
  });
}

test('Crew translations react to language selection and finished cards cannot be played', async ({
  page,
}) => {
  const room = fixture();
  const { publish, moves } = await mock(page, room);
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByLabel('Language', { exact: true }).selectOption('de-cool');
  await page.keyboard.press('Escape');
  await expect(
    page.getByRole('button', { name: 'Blau 1 auswählen', exact: true }),
  ).toBeVisible();
  await page
    .getByRole('button', { name: 'Blau 1 auswählen', exact: true })
    .click();
  room.game!.outcome = { winners: [], scores: null };
  room.game!.legal_actions = [];
  room.revision++;
  publish();
  await expect(
    page.getByRole('button', { name: 'Karte spielen', exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole('button', { name: 'Blau 1 auswählen', exact: true }),
  ).toBeDisabled();
  expect(moves).toHaveLength(0);
});

test('Crew touch controls select then play and open opponent info', async ({
  browser,
}) => {
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    isMobile: true,
    hasTouch: true,
  });
  const page = await context.newPage();
  const { moves } = await mock(page, fixture());
  await page.getByRole('button', { name: 'Info about Bob', exact: true }).tap();
  await page.getByRole('button', { name: 'Close', exact: true }).tap();
  await page.getByRole('button', { name: 'Select Blue 1', exact: true }).tap();
  expect(moves).toHaveLength(0);
  await page.getByRole('button', { name: 'Play card', exact: true }).tap();
  await expect
    .poll(() => moves)
    .toEqual([{ game: 'the_crew', action: { type: 'play', index: 0 } }]);
  await context.close();
});
