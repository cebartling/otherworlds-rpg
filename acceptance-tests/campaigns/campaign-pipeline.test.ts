import { test, expect, type Page } from '@playwright/test';

const VALID_CAMPAIGN_SOURCE = `---
title: Test Campaign
description: A test campaign for acceptance testing
---

# Act 1: The Beginning

## Scene 1: Arrival

The adventurers arrive at the village square.
`;

const INVALID_CAMPAIGN_SOURCE = 'This has no YAML front-matter and no scenes.';

/**
 * Helper: ingest a campaign via the textarea form and return the new campaign ID.
 */
async function ingestCampaign(page: Page, source: string): Promise<string> {
  await page.goto('/campaigns');

  // Open the ingest form
  await page.getByRole('button', { name: 'Ingest Campaign' }).click();

  // Fill and submit
  await page.locator('#campaign-source').fill(source);
  await page.getByRole('button', { name: 'Ingest' }).click();

  // Wait for redirect to /campaigns/<uuid>
  await page.waitForURL(/\/campaigns\/[0-9a-f-]{36}$/);

  const url = page.url();
  const campaignId = url.split('/campaigns/')[1];
  return campaignId;
}

test.describe('Campaign Pipeline', () => {

  test('ingest campaign via textarea form', async ({ page }) => {
    const campaignId = await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Should be on the detail page
    expect(campaignId).toMatch(/^[0-9a-f-]{36}$/);

    // Pipeline step 1 (Ingested) should be green
    const ingestedStep = page.locator('text=Ingested').first();
    await expect(ingestedStep).toBeVisible();

    // Version should show v1
    await expect(page.getByText('v1')).toBeVisible();
  });

  test('validate ingested campaign', async ({ page }) => {
    await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Click Validate
    await page.getByRole('button', { name: 'Validate' }).click();

    // Wait for form submission to complete
    await expect(page.getByText('Campaign validated successfully.')).toBeVisible();

    // Validate button should now be disabled
    await expect(page.getByRole('button', { name: 'Validate' })).toBeDisabled();

    // Compile button should now be enabled
    await expect(page.getByRole('button', { name: 'Compile' })).toBeEnabled();
  });

  test('compile validated campaign', async ({ page }) => {
    await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Validate first
    await page.getByRole('button', { name: 'Validate' }).click();
    await expect(page.getByText('Campaign validated successfully.')).toBeVisible();

    // Now compile
    await page.getByRole('button', { name: 'Compile' }).click();
    await expect(page.getByText('Campaign compiled successfully.')).toBeVisible();

    // Both buttons should be disabled
    await expect(page.getByRole('button', { name: 'Validate' })).toBeDisabled();
    await expect(page.getByRole('button', { name: 'Compile' })).toBeDisabled();
  });

  test('archive campaign removes from list', async ({ page }) => {
    const campaignId = await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Click Archive, then Confirm Archive
    await page.getByRole('button', { name: 'Archive' }).click();
    await page.getByRole('button', { name: 'Confirm Archive' }).click();

    // Should redirect to /campaigns
    await page.waitForURL('/campaigns');

    // Campaign should no longer appear in the list
    await expect(page.locator(`a[href="/campaigns/${campaignId}"]`)).toHaveCount(0);
  });

  test('campaigns list shows ingested campaigns', async ({ page }) => {
    await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Navigate to campaigns list
    await page.goto('/campaigns');

    // At least one campaign card should be visible with an Ingested badge
    const ingestedBadge = page.locator('text=Ingested').first();
    await expect(ingestedBadge).toBeVisible();
  });

  test('validation error displays inline', async ({ page }) => {
    await ingestCampaign(page, INVALID_CAMPAIGN_SOURCE);

    // Click Validate — should produce a client error
    await page.getByRole('button', { name: 'Validate' }).click();

    // Error should appear inline (not a SvelteKit error page)
    // Check that we're still on the campaign detail page, not an error page
    await expect(page).toHaveURL(/\/campaigns\/[0-9a-f-]{36}$/);

    // The error message should be visible in the inline error block
    const errorBlock = page.locator('[style*="rgba(198, 40, 40"]');
    await expect(errorBlock).toBeVisible();
  });

});
