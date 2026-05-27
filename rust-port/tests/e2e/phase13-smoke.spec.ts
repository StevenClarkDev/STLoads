import { expect, test } from '@playwright/test';

const routes = [
  { path: '/', label: /STLoads|LoadBoard|Log In/i },
  { path: '/auth/login', label: /Login|Email|Password/i },
  { path: '/dashboard', label: /Dashboard|Checking session|Sign in/i },
  { path: '/loads', label: /Load|Sign in|Checking session/i },
  { path: '/notifications', label: /Notification|Sign in|Checking session/i },
];

test.describe('Phase 13 frontend smoke coverage', () => {
  for (const route of routes) {
    test(`loads ${route.path}`, async ({ page }) => {
      const consoleErrors: string[] = [];
      page.on('console', (message) => {
        if (message.type() === 'error') {
          consoleErrors.push(message.text());
        }
      });

      await page.goto(route.path, { waitUntil: 'domcontentloaded' });
      await expect(page.locator('body')).toContainText(route.label);
      await expect(page.locator('body')).toBeVisible();
      expect(consoleErrors.filter((line) => !line.includes('runtime-config.js'))).toEqual([]);
    });
  }
});

test.describe('Phase 13 visual checkpoints', () => {
  test('login page visual baseline', async ({ page }) => {
    await page.goto('/auth/login', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('body')).toContainText(/Login|Email/i);
    await expect(page).toHaveScreenshot('login-page.png', { fullPage: true });
  });

  test('public landing visual baseline', async ({ page }) => {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('body')).toContainText(/STLoads|LoadBoard|Log In/i);
    await expect(page).toHaveScreenshot('public-landing.png', { fullPage: true });
  });
});
