import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

test('landing has only entry controls, accessible hints, and no heading', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.locator('header, h1')).toHaveCount(0);
  await expect(page.getByRole('textbox')).toHaveCount(2);
  await expect(page.getByPlaceholder('Nickname')).toHaveAccessibleName(
    'Nickname',
  );
  await expect(page.getByPlaceholder('Room code')).toHaveAccessibleName(
    'Room code',
  );
  await expect(
    page.getByRole('button', { name: 'Create room' }),
  ).toBeDisabled();
  await page.getByLabel('Nickname').fill('Alice');
  await expect(page.getByRole('button', { name: 'Create room' })).toBeEnabled();
  await page.getByLabel('Room code').fill('ABC01!');
  await expect(page.getByRole('button', { name: 'Join room' })).toBeDisabled();
  await page.getByLabel('Room code').fill('abc234');
  await expect(page.getByRole('button', { name: 'Join room' })).toBeEnabled();
});

test('device German and explicit cool selection persist and retain values', async ({
  browser,
}) => {
  const context = await browser.newContext({ locale: 'de-AT' });
  const page = await context.newPage();
  await page.goto('/');
  await expect(page.locator('html')).toHaveAttribute('lang', 'de');
  await page.getByLabel('Spitzname').fill('Alice');
  await page.getByRole('button', { name: 'Einstellungen' }).click();
  await page.getByLabel('Sprache', { exact: true }).selectOption('de-cool');
  await page.keyboard.press('Escape');
  await expect(page.getByLabel('Sprache', { exact: true })).toBeHidden();
  await page.reload();
  await expect(page.getByLabel('Spitzname')).toBeVisible();
  await page.getByRole('button', { name: 'Einstellungen' }).click();
  await expect(page.getByLabel('Sprache', { exact: true })).toHaveValue(
    'de-cool',
  );
  await page.getByLabel('Sprache', { exact: true }).selectOption('en');
  await expect(page.getByLabel('Nickname')).toBeVisible();
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await page.getByLabel('Nickname').click();
  await expect(page.getByLabel('Language', { exact: true })).toBeHidden();
  await page.getByLabel('Nickname').fill('Bob');
  await page.getByRole('button', { name: 'Settings' }).click();
  await page.getByLabel('Language', { exact: true }).selectOption('de');
  await expect(page.getByLabel('Spitzname')).toHaveValue('Bob');
  await context.close();
});

test('device language selection skips unsupported preferences in order', async ({
  page,
}) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'languages', {
      value: ['fr-FR', 'de-DE', 'en-US'],
    });
    localStorage.setItem('arena-language', 'invalid');
  });
  await page.goto('/');
  await expect(page.getByLabel('Spitzname')).toBeVisible();
});

test('unsupported device locale falls back to English', async ({ browser }) => {
  const context = await browser.newContext({ locale: 'fr-FR' });
  const page = await context.newPage();
  await page.goto('/');
  await expect(page.getByLabel('Nickname')).toBeVisible();
  await context.close();
});

test('independent catalogs have matching keys', async () => {
  const catalogs = await Promise.all(
    ['en', 'de', 'de-cool'].map(async (name) =>
      JSON.parse(await readFile(`src/lib/locales/${name}.json`, 'utf8')),
    ),
  );
  function keys(value: Record<string, unknown>): unknown {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [
        key,
        typeof item === 'object' && item !== null
          ? keys(item as Record<string, unknown>)
          : true,
      ]),
    );
  }
  expect(keys(catalogs[0])).toEqual(keys(catalogs[1]));
  expect(keys(catalogs[1])).toEqual(keys(catalogs[2]));
});

for (const [name, width, height] of [
  ['mobile', 390, 844],
  ['tablet', 768, 1024],
  ['desktop', 1280, 800],
  ['narrow', 320, 568],
] as const) {
  test(`${name}: responsive panels, settings, local fonts, and no overflow`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height });
    const external: string[] = [];
    page.on('request', (request) => {
      if (!request.url().startsWith('http://localhost:15173'))
        external.push(request.url());
    });
    await page.goto('/');
    const create = (await page
      .getByRole('button', { name: 'Create room' })
      .boundingBox())!;
    const join = (await page
      .getByRole('button', { name: 'Join room' })
      .boundingBox())!;
    const nickname = (await page.getByLabel('Nickname').boundingBox())!;
    const settings = (await page
      .getByRole('button', { name: 'Settings' })
      .boundingBox())!;
    expect(nickname.y + nickname.height).toBeLessThan(create.y);
    if (width > 640) {
      expect(Math.abs(create.y - join.y)).toBeLessThan(2);
      expect(join.x).toBeGreaterThan(create.x + create.width);
      expect(settings.y).toBeLessThan(height / 2);
    } else {
      expect(join.y).toBeGreaterThan(create.y + create.height);
      expect(settings.y).toBeGreaterThan(height / 2);
    }
    for (const box of [create, join, nickname, settings]) {
      expect(box.height).toBeGreaterThanOrEqual(44);
      expect(box.x + box.width).toBeLessThanOrEqual(width);
    }
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
    ).toBe(width);
    await page.evaluate(() => document.fonts.ready);
    expect(
      await page.evaluate(() =>
        document.fonts.check('16px "Nunito Sans Variable"'),
      ),
    ).toBe(true);
    expect(external).toEqual([]);
  });
}

test('invitation code remains prefilled and failed entry is translated', async ({
  page,
}) => {
  await page.route('**/api/rooms/ABC234/join', (route) =>
    route.fulfill({ status: 404, json: { error: 'Room or replay not found' } }),
  );
  await page.goto('/room/ABC234');
  await expect(page.getByRole('button', { name: 'Create room' })).toHaveCount(
    0,
  );
  await expect(page.getByLabel('Room code')).toHaveValue('ABC234');
  await page.getByLabel('Nickname').fill('Alice');
  await page.getByRole('button', { name: 'Join room' }).click();
  await expect(page.getByRole('alert')).toHaveText('Room not found');
  await page.getByRole('button', { name: 'Settings' }).click();
  await page.getByLabel('Language', { exact: true }).selectOption('de');
  await expect(page.getByRole('alert')).toHaveText('Raum nicht gefunden');
});

test('entry failures distinguish nickname problems and use a translated fallback', async ({
  page,
}) => {
  let message = 'Nickname already taken';
  await page.route('**/api/rooms', (route) =>
    route.fulfill({ status: 400, json: { error: message } }),
  );
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).click();
  await page.getByLabel('Language', { exact: true }).selectOption('de');
  await page.keyboard.press('Escape');
  await page.getByLabel('Spitzname').fill('Alice');
  await page.getByRole('button', { name: 'Raum erstellen' }).click();
  await expect(page.getByRole('alert')).toHaveText(
    'Spitzname bereits vergeben',
  );
  message = 'Use a nickname of 1–24 characters';
  await page.getByRole('button', { name: 'Raum erstellen' }).click();
  await expect(page.getByRole('alert')).toHaveText(
    'Verwende einen Spitznamen mit 1–24 Zeichen',
  );
  message = 'Storage error';
  await page.getByRole('button', { name: 'Raum erstellen' }).click();
  await expect(page.getByRole('alert')).toHaveText(
    'Beitritt zum Raum fehlgeschlagen',
  );
});
