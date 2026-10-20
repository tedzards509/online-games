import { test, expect } from '@playwright/test';

test('card layers share their shape, scale content, and expose accessible names', async ({
  page,
}) => {
  await page.goto('/tests/fixtures/cards.html');
  const moon = page.getByRole('img', { name: 'Custom moon' });
  await expect(moon.locator('svg')).toHaveCount(1);
  await expect(page.getByRole('img', { name: 'Ace of hearts' })).toBeVisible();
  await expect(
    page.getByRole('img', { name: 'Queen of diamonds' }),
  ).toBeVisible();
  await expect(page.getByRole('img', { name: '10 of spades' })).toBeVisible();
  await expect(
    page.getByRole('img', { name: 'Face-down card' }),
  ).not.toContainText(/[AKQJ0-9]/);
  const star = page.getByRole('img', { name: 'Purple star' });
  await expect(star).toHaveCSS('color', 'rgb(102, 51, 153)');
  await expect(star).toHaveCSS('background-color', 'rgb(230, 230, 250)');
  const cards = page.locator('.playing-card');
  for (const card of await cards.all()) {
    const box = (await card.boundingBox())!;
    expect(box.width / box.height).toBeCloseTo(5 / 7, 2);
  }
  const ace = page.getByRole('img', { name: 'Ace of hearts' });
  await expect(ace).toHaveCSS('color', 'rgb(182, 64, 50)');
  await expect(page.getByRole('img', { name: '10 of spades' })).toHaveCSS(
    'color',
    'rgb(33, 81, 60)',
  );
  const largeSymbol = await ace.locator('.center').boundingBox();
  await ace.evaluate((node) => {
    node.style.width = '50px';
  });
  const smallSymbol = await ace.locator('.center').boundingBox();
  expect(smallSymbol!.width).toBeLessThan(largeSymbol!.width * 0.6);
});

test('only interactive cards take focus; selection, disabled state, and translations work', async ({
  page,
}) => {
  await page.goto('/tests/fixtures/cards.html');
  const king = page.getByRole('button', { name: 'King of clubs' });
  await expect(king).toHaveAttribute('aria-pressed', 'false');
  await expect(
    page.getByRole('button', { name: 'Disabled card' }),
  ).toBeDisabled();
  await page.keyboard.press('Tab');
  await expect(king).toBeFocused();
  await page.keyboard.press('Space');
  await expect(king).toHaveAttribute('aria-pressed', 'true');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('button', { name: 'German' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('img', { name: 'Herz Ass' })).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Kreuz König' }),
  ).toHaveAttribute('aria-pressed', 'true');
  await expect(
    page.getByRole('img', { name: 'Verdeckte Karte' }),
  ).toBeVisible();
});
