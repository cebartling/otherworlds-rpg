import { test, expect } from '@playwright/test';

test('home page loads with expected content', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('h1')).toHaveText('Welcome to SvelteKit');
});

test('home page has a link to SvelteKit docs', async ({ page }) => {
  await page.goto('/');
  const docsLink = page.locator('a[href="https://svelte.dev/docs/kit"]');
  await expect(docsLink).toBeVisible();
});
