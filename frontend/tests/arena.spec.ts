import { test, expect, type Page } from '@playwright/test';
import { readFile } from 'node:fs/promises';

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
