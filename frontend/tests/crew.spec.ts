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
  const commands: {
    type: string;
    message?: { type: string; won?: boolean };
  }[] = [];
  let publish: (acknowledged?: boolean) => void = () => {};
  let reject = () => {};
  let disconnect = () => {};
  let online = true;
  await page.routeWebSocket('**/api/rooms/ABC234/ws', (socket) => {
    reject = () =>
      socket.send(JSON.stringify({ type: 'error', error: 'Rejected move' }));
    disconnect = () => {
      online = false;
      socket.close();
    };
    publish = (acknowledged = false) =>
      socket.send(JSON.stringify({ type: 'snapshot', room, acknowledged }));
    socket.onMessage((data) => {
      const message = JSON.parse(String(data));
      if (message.type !== 'authenticate') commands.push(message);
      if (message.type === 'game') {
        if (message.message.type === 'move') moves.push(message.message.action);
        return;
      }
      if (message.type === 'authenticate' && online) publish();
    });
  });
  await page.goto('/room/ABC234');
  await expect(
    page.getByRole('button', { name: 'Play card', exact: true }),
  ).toBeVisible();
  return {
    moves,
    commands,
    publish: (acknowledged = false) => publish(acknowledged),
    reject: () => reject(),
    disconnect: () => disconnect(),
  };
}

test('Crew keeps card selection opaque and retains it through pending and rejected moves', async ({
  page,
}) => {
  const room = fixture();
  const { moves, publish, reject } = await mock(page, room);
  const play = page.getByRole('button', { name: 'Play card', exact: true });
  await expect(play).toBeDisabled();
  const illegal = page.getByRole('button', {
    name: 'Select Green 4',
    exact: true,
  });
  await illegal.click();
  await expect(illegal).toHaveAttribute('aria-pressed', 'true');
  await expect(illegal.locator('..')).toHaveAttribute('data-muted', 'true');
  await expect(illegal).toHaveCSS('opacity', '1');
  await expect(illegal.locator('..')).toHaveCSS('opacity', '1');
  await expect(play).toBeDisabled();
  const blue = page.getByRole('button', { name: 'Select Blue 6', exact: true });
  await blue.click();
  expect(moves).toHaveLength(0);
  await expect(blue).toHaveAttribute('aria-pressed', 'true');
  await play.click();
  await expect
    .poll(() => moves)
    .toEqual([{ game: 'the_crew', action: { type: 'play', index: 1 } }]);
  await expect(blue).toBeEnabled();
  await expect(play).toBeDisabled();
  room.revision++;
  publish();
  await expect(play).toBeDisabled();
  reject();
  await expect(play).toBeEnabled();
  await expect(blue).toHaveAttribute('aria-pressed', 'true');
  expect(moves).toHaveLength(1);
  await play.click();
  view(room).hand.splice(1, 1);
  publish(true);
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
  const handBefore = await page
    .getByRole('region', { name: 'Your hand', exact: true })
    .boundingBox();
  await page
    .getByRole('button', { name: 'Communicate a card', exact: true })
    .click();
  expect(
    await page
      .getByRole('region', { name: 'Your hand', exact: true })
      .boundingBox(),
  ).toEqual(handBefore);
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
  const room = fixture();
  room.game!.legal_actions = view(room).hand.map((_, index) => ({
    game: 'the_crew',
    action: { type: 'play', index },
  }));
  await mock(page, room);
  for (const [name, color, shape] of [
    ['Blue 1', 'rgb(0, 139, 207)', 'circle'],
    ['Green 4', 'rgb(140, 198, 62)', 'path'],
    ['Pink 7', 'rgb(231, 124, 177)', 'rect'],
    ['Yellow 9', 'rgb(223, 174, 65)', 'path'],
  ]) {
    const face = page.getByRole('button', {
      name: `Select ${name}`,
      exact: true,
    });
    await expect(face).toHaveCSS('background-color', color);
    await expect(face.locator('svg').first()).toHaveAttribute('fill', 'none');
    await expect(face.locator(`svg ${shape}`).first()).toBeAttached();
  }
  await expect(
    page
      .getByRole('button', { name: 'Select Yellow 9', exact: true })
      .locator('svg path')
      .first(),
  ).toHaveAttribute('d', 'M5 5 27 27M27 5 5 27');
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
  for (const slot of [bob, carol]) {
    const box = (await slot.boundingBox())!;
    const marker = (await slot.locator('.marker').boundingBox())!;
    expect(marker.width).toBeCloseTo(box.width * 0.5, 0);
    expect(marker.height).toBeCloseTo(marker.width, 0);
    expect(
      Math.abs(marker.x + marker.width / 2 - (box.x + box.width / 2)),
    ).toBeLessThan(1);
    expect(marker.y).toBeGreaterThanOrEqual(box.y);
    expect(marker.y + marker.height).toBeLessThanOrEqual(box.y + box.height);
    if (slot === carol) {
      expect(marker.y + marker.height / 2).toBeCloseTo(
        box.y + box.height / 2,
        0,
      );
    }
  }
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

for (const commander of [0, 1]) {
  test(`Crew commander plaque scales to the card width for seat ${commander}`, async ({
    page,
  }) => {
    const room = fixture();
    view(room).commander = commander;
    await mock(page, room);
    const plaque = page.getByRole('img', {
      name: 'Commander · original black-four holder',
      exact: true,
    });
    await expect(plaque).toBeVisible();
    const plaqueBox = (await plaque.boundingBox())!;
    const cardBox = (await page
      .locator('.communication')
      .first()
      .boundingBox())!;
    expect(plaqueBox.width).toBeCloseTo(cardBox.width * 0.75, 0);
    expect(plaqueBox.height).toBeCloseTo(plaqueBox.width, 0);
  });
}

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
  await expect(opponent.locator('.backs .playing-card')).toHaveCount(8);
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

for (const width of [1280, 390]) {
  test(`Crew previous-trick viewer shows full-size cards and owners at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    const room = fixture();
    view(room).last_trick = {
      winner: 4,
      cards: room.game!.players.map((_, player) => ({
        player,
        card: { color: 'blue', number: player + 1 },
      })),
    };
    view(room).trick = [{ player: 4, card: { color: 'yellow', number: 7 } }];
    await mock(page, room);
    const hand = page.getByRole('region', { name: 'Your hand', exact: true });
    const before = await hand.boundingBox();
    await page.getByRole('button', { name: 'Last trick', exact: true }).click();
    const viewer = page.getByRole('region', {
      name: 'Last trick',
      exact: true,
    });
    await expect(viewer.getByRole('img')).toHaveCount(5);
    await expect(viewer.locator('figcaption')).toHaveText(
      room.game!.players.map((player) => player.nickname),
    );
    await expect(viewer).toContainText('Erin won the trick');
    for (const card of await viewer.locator('.playing-card').all())
      await expect(card).toHaveCSS('width', '72px');
    expect(await hand.boundingBox()).toEqual(before);
    await page.keyboard.press('Escape');
    await expect(viewer).toBeHidden();
    expect(
      await page.evaluate(() => document.documentElement.scrollHeight),
    ).toBe(844);
  });
}

test('Crew shows a localized turn pill only for the actual next player', async ({
  page,
}) => {
  const room = fixture();
  const { publish } = await mock(page, room);
  const status = page.locator('.crew-table .status');
  await expect(status).toHaveText('Your turn');
  await expect(status).toHaveClass(/your-turn/);
  await expect(status).toHaveCSS('background-color', 'rgb(35, 94, 69)');
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByLabel('Language', { exact: true }).selectOption('de');
  await page.keyboard.press('Escape');
  await expect(status).toHaveText('Du bist dran');
  // Eligibility for communication does not make it this viewer's card-playing turn.
  view(room).next_player = 1;
  room.revision++;
  publish();
  await expect(status).toHaveText('Bob spielt aus');
  await expect(status).not.toHaveClass(/your-turn/);
  view(room).next_player = 0;
  view(room).trick = [{ player: 4, card: { color: 'blue', number: 9 } }];
  room.revision++;
  publish();
  await expect(status).toHaveText('Du bist dran');
  await expect(status).toHaveClass(/your-turn/);
  view(room).awaiting_result = true;
  view(room).trick = [];
  room.revision++;
  publish();
  await expect(status).not.toHaveClass(/your-turn/);
});

for (const [width, height, count] of [
  [1280, 900, 5],
  [768, 1024, 4],
  [390, 844, 5],
  [320, 568, 3],
  [320, 568, 5],
  [844, 390, 5],
] as const) {
  test(`Crew ${count} players at ${width}×${height} keeps a viewport-sized table and straight card row`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height });
    const room = fixture(count, 1);
    view(room).last_trick = {
      winner: 0,
      cards: room.game!.players.map((_, player) => ({
        player,
        card: { color: 'blue', number: player + 1 },
      })),
    };
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
    view(room).communications[0] = {
      card: { color: 'pink', number: 3 },
      declaration: 'only',
    };
    view(room).communications[1] = {
      card: view(room).hand[0],
      declaration: 'lowest',
    };
    view(room).tricks_won[0] = 2;
    view(room).tricks_won[1] = 1;
    await mock(page, room);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
    ).toBe(width);
    expect(
      await page.evaluate(() => document.documentElement.scrollHeight),
    ).toBe(height);
    await expect(page.locator('header, h1:visible')).toHaveCount(0);
    expect((await page.locator('main').boundingBox())!.height).toBe(height);
    const hand = page.getByRole('region', { name: 'Your hand', exact: true });
    const row = hand.locator('.card-row');
    await expect(row.locator('button')).toHaveCount(view(room).hand.length);
    for (const card of await row.locator('button').all())
      await expect(card).toHaveCSS('transform', 'none');
    const boxes = await row.locator('button').evaluateAll((cards) =>
      cards.map((card) => {
        const box = card.getBoundingClientRect();
        return { x: box.x, right: box.right, y: box.y };
      }),
    );
    for (let index = 1; index < boxes.length; index++) {
      expect(boxes[index].x).toBeGreaterThan(boxes[index - 1].right);
      expect(boxes[index].y).toBeCloseTo(boxes[0].y, 1);
    }
    const playBox = (await page
      .getByRole('button', { name: 'Play card', exact: true })
      .boundingBox())!;
    expect(playBox.y + playBox.height).toBeLessThanOrEqual(height);
    const statusBox = (await page
      .locator('.crew-table .status')
      .boundingBox())!;
    const arenaBox = (await page.locator('.arena').boundingBox())!;
    expect(arenaBox.y + arenaBox.height).toBeLessThanOrEqual(statusBox.y);
    const sizes = await page
      .locator('.crew-table .playing-card:visible')
      .evaluateAll((cards) =>
        cards.map((card) => {
          const style = getComputedStyle(card);
          return {
            width: parseFloat(style.width),
            height: parseFloat(style.height),
          };
        }),
      );
    expect(sizes.length).toBeGreaterThan(view(room).hand.length);
    for (const size of sizes) {
      expect(size.width).toBe(72);
      expect(size.height).toBeCloseTo(72 * 1.4, 1);
    }
    const handBox = (await hand.boundingBox())!;
    const communicationBox = (await page
      .locator('.own-communication')
      .boundingBox())!;
    const pileBox = (await page.locator('.own-tricks').boundingBox())!;
    if (width <= 640 && height > width) {
      expect(communicationBox.y + communicationBox.height).toBeLessThanOrEqual(
        handBox.y,
      );
      expect(pileBox.y + pileBox.height).toBeLessThanOrEqual(handBox.y);
      expect(handBox.width).toBe(width - 16);
    } else {
      expect(communicationBox.x + communicationBox.width).toBeLessThanOrEqual(
        handBox.x,
      );
      expect(pileBox.x).toBeGreaterThanOrEqual(handBox.x + handBox.width);
    }
    const lastTrickCard = page.locator('.trick .playing-card').last();
    await lastTrickCard.scrollIntoViewIfNeeded();
    await expect(lastTrickCard).toBeInViewport();
    expect(await hand.boundingBox()).toEqual(handBox);
    await row.locator('button').last().focus();
    await page.keyboard.press('Space');
    await expect(row.locator('button').last()).toHaveAttribute(
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
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
    ).toBe(width);
    expect(
      await page.evaluate(() => document.documentElement.scrollHeight),
    ).toBe(height);
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
  ).toBeEnabled();
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

test('Crew cards remain fully colored and selectable outside the viewer turn', async ({
  page,
}) => {
  const room = fixture();
  view(room).next_player = 1;
  room.game!.legal_actions = [
    {
      game: 'the_crew',
      action: { type: 'communicate', index: 2, declaration: 'only' },
    },
  ];
  const { moves } = await mock(page, room);
  await expect(
    page.locator('.hand-scroll .crew-card[data-muted="true"]'),
  ).toHaveCount(0);
  for (const card of await page.locator('.hand-scroll button').all()) {
    await expect(card).toBeEnabled();
    await expect(card).toHaveCSS('opacity', '1');
  }
  await page
    .getByRole('button', { name: 'Select Green 4', exact: true })
    .click();
  await expect(
    page.getByRole('button', { name: 'Play card', exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole('button', { name: 'Communicate a card', exact: true }),
  ).toBeEnabled();
  expect(moves).toHaveLength(0);
});

for (const width of [1280, 390]) {
  test(`Crew host controls mirror settings at ${width}px without consuming playing space`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    const room = fixture(5, 1);
    room.game!.can_finish = true;
    const { commands, publish } = await mock(page, room);
    const trigger = page.getByRole('button', {
      name: 'Room controls',
      exact: true,
    });
    const settings = (await page
      .getByRole('button', { name: 'Settings', exact: true })
      .boundingBox())!;
    const controls = (await trigger.boundingBox())!;
    expect(controls.y).toBeCloseTo(settings.y, 1);
    expect(controls.x).toBeCloseTo(width - settings.x - settings.width, 1);
    await expect(
      page.getByRole('button', { name: 'Abort game', exact: true }),
    ).toBeHidden();
    const before = await page.locator('.crew-table').boundingBox();
    await trigger.click();
    expect(await page.locator('.crew-table').boundingBox()).toEqual(before);
    await page
      .getByRole('button', { name: 'Finish: lost', exact: true })
      .click();
    await expect
      .poll(() => commands.at(-1)?.message)
      .toEqual({ type: 'finish', won: false });
    room.game!.outcome = { winners: [], scores: null };
    room.game!.can_finish = false;
    room.game!.legal_actions = [];
    room.revision++;
    publish(true);
    await page.getByRole('button', { name: 'Close', exact: true }).click();
    await expect(page.locator('.crew-table').getByRole('status')).toHaveText(
      'Lost',
    );
    await trigger.click();
    await page
      .getByRole('button', { name: 'Return to setup', exact: true })
      .click();
    await expect.poll(() => commands.at(-1)?.type).toEqual('abort');
    room.game = null;
    room.revision++;
    publish(true);
    await expect(page.locator('header')).toBeVisible();
    await expect(page.getByLabel('Members', { exact: true })).toBeVisible();
    await expect(page.locator('main')).not.toHaveClass(/crew-shell/);
  });
}

test('Crew guests have no host controls and disconnected selection cannot submit', async ({
  page,
}) => {
  const room = fixture();
  const { moves, disconnect } = await mock(page, room);
  await expect(
    page.getByRole('button', { name: 'Room controls', exact: true }),
  ).toHaveCount(0);
  await page
    .getByRole('button', { name: 'Select Blue 1', exact: true })
    .click();
  disconnect();
  await expect(page.locator('.connection-notice')).toHaveText('Reconnecting…');
  await expect(
    page.getByRole('button', { name: 'Play card', exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole('button', { name: 'Select Blue 1', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  expect(moves).toHaveLength(0);
});
