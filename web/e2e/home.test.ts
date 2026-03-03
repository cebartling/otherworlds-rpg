import { test, expect } from '@playwright/test';

test('home page loads with expected content', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('h1')).toContainText('Otherworlds RPG');
});

test('home page has navigation links', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('nav a[href="/campaigns"]')).toBeVisible();
  await expect(page.locator('nav a[href="/characters"]')).toBeVisible();
  await expect(page.locator('nav a[href="/sessions"]')).toBeVisible();
});
