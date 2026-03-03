import { test, expect, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const VALID_CAMPAIGN_SOURCE = readFileSync(
  resolve(__dirname, '../fixtures/the-lost-temple.md'),
  'utf-8',
);

const INVALID_CAMPAIGN_SOURCE = 'This has no YAML front-matter and no scenes.';

/** RegExp matching the green background style (hex from SSR, rgb from client-side). */
const GREEN_STYLE = /background-color: (#2e7d32|rgb\(46, 125, 50\))/;
/** RegExp matching the gray border style on inactive pipeline steps/badges. */
const GRAY_STYLE = /border: 1px solid var\(--color-border\)/;

/**
 * Helper: ingest a campaign via the textarea form and return the new campaign ID.
 */
async function ingestCampaign(page: Page, source: string): Promise<string> {
  await page.goto('/campaigns', { waitUntil: 'networkidle' });

  // Open the ingest form (toggle button reveals the textarea)
  const toggleButton = page.getByRole('button', { name: 'Ingest Campaign' });
  await toggleButton.click();

  // Wait for the button text to change to "Cancel" (confirms JS hydration + toggle)
  await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible({ timeout: 10_000 });

  // Fill and submit
  const textarea = page.locator('#campaign-source');
  await textarea.fill(source);
  await page.getByRole('button', { name: 'Ingest', exact: true }).click();

  // Wait for redirect to /campaigns/<uuid>
  await page.waitForURL(/\/campaigns\/[0-9a-f-]{36}$/);

  const url = page.url();
  const campaignId = url.split('/campaigns/')[1];
  return campaignId;
}

/**
 * Returns the pipeline step circle (the numbered div) for a given step number.
 * Steps are 1=Ingested, 2=Validated, 3=Compiled.
 */
function pipelineStepCircle(page: Page, stepNumber: number) {
  return page
    .locator('section')
    .filter({ hasText: 'Content Pipeline' })
    .locator('.rounded-full')
    .filter({ hasText: String(stepNumber) })
    .first();
}

test.describe('Campaign Pipeline', () => {

  test('ingest campaign via textarea form', async ({ page }) => {
    const campaignId = await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Should be on the detail page with a valid UUID
    expect(campaignId).toMatch(/^[0-9a-f-]{36}$/);

    // The heading should display the campaign ID prefix
    const heading = page.locator('h1');
    await expect(heading).toBeVisible();
    const headingText = await heading.textContent();
    expect(headingText).toContain(campaignId.substring(0, 8));

    // Pipeline step 1 (Ingested) should be green
    const step1 = pipelineStepCircle(page, 1);
    await expect(step1).toHaveAttribute('style', GREEN_STYLE);

    // Pipeline steps 2 and 3 should be gray (not yet reached)
    const step2 = pipelineStepCircle(page, 2);
    await expect(step2).toHaveAttribute('style', GRAY_STYLE);
    const step3 = pipelineStepCircle(page, 3);
    await expect(step3).toHaveAttribute('style', GRAY_STYLE);

    // Version should show v1 in the Version Details section
    const versionSection = page.locator('section').filter({ hasText: 'Version Details' });
    await expect(versionSection.locator('p', { hasText: 'v1' })).toBeVisible();

    // Version hash should be present (not N/A)
    const hashText = await versionSection.locator('p.font-mono').textContent();
    expect(hashText).not.toBe('N/A');
    expect(hashText!.length).toBeGreaterThan(10);

    // Validate button should be enabled (ingested but not validated)
    await expect(page.getByRole('button', { name: 'Validate' })).toBeEnabled();

    // Compile button should be disabled (not yet validated)
    await expect(page.getByRole('button', { name: 'Compile' })).toBeDisabled();
  });

  test('validate ingested campaign', async ({ page }) => {
    await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Click Validate
    await page.getByRole('button', { name: 'Validate' }).click();

    // Wait for form submission to complete
    await expect(page.getByText('Campaign validated successfully.')).toBeVisible();

    // Pipeline steps 1 and 2 should now be green
    const step1 = pipelineStepCircle(page, 1);
    await expect(step1).toHaveAttribute('style', GREEN_STYLE);
    const step2 = pipelineStepCircle(page, 2);
    await expect(step2).toHaveAttribute('style', GREEN_STYLE);

    // Pipeline step 3 should still be gray
    const step3 = pipelineStepCircle(page, 3);
    await expect(step3).toHaveAttribute('style', GRAY_STYLE);

    // Validate button should now be disabled (already validated)
    await expect(page.getByRole('button', { name: 'Validate' })).toBeDisabled();

    // Compile button should now be enabled (validated but not compiled)
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

    // All three pipeline steps should be green
    const step1 = pipelineStepCircle(page, 1);
    await expect(step1).toHaveAttribute('style', GREEN_STYLE);
    const step2 = pipelineStepCircle(page, 2);
    await expect(step2).toHaveAttribute('style', GREEN_STYLE);
    const step3 = pipelineStepCircle(page, 3);
    await expect(step3).toHaveAttribute('style', GREEN_STYLE);

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

  test('campaigns list shows ingested campaign with correct badge state', async ({ page }) => {
    const campaignId = await ingestCampaign(page, VALID_CAMPAIGN_SOURCE);

    // Navigate to campaigns list
    await page.goto('/campaigns');

    // The specific campaign card should be present (link to its detail page)
    const campaignCard = page.locator(`a[href="/campaigns/${campaignId}"]`);
    await expect(campaignCard).toBeVisible();

    // The Ingested badge on this card should be green-styled
    const ingestedBadge = campaignCard.locator('span').filter({ hasText: 'Ingested' });
    await expect(ingestedBadge).toHaveAttribute('style', GREEN_STYLE);

    // The Validated badge should be gray (not green)
    const validatedBadge = campaignCard.locator('span').filter({ hasText: 'Validated' });
    await expect(validatedBadge).toHaveAttribute('style', GRAY_STYLE);

    // The Compiled badge should be gray (not green)
    const compiledBadge = campaignCard.locator('span').filter({ hasText: 'Compiled' });
    await expect(compiledBadge).toHaveAttribute('style', GRAY_STYLE);
  });

  test('validation error displays inline', async ({ page }) => {
    await ingestCampaign(page, INVALID_CAMPAIGN_SOURCE);

    // Click Validate — should produce a client error
    await page.getByRole('button', { name: 'Validate' }).click();

    // Should still be on the campaign detail page (not a SvelteKit error page)
    await expect(page).toHaveURL(/\/campaigns\/[0-9a-f-]{36}$/);

    // The inline error block should be visible with an actual error message
    const errorBlock = page.locator('[style*="rgba(198, 40, 40"]');
    await expect(errorBlock).toBeVisible();
    const errorText = await errorBlock.textContent();
    expect(errorText!.length).toBeGreaterThan(0);

    // Success message should NOT be visible
    await expect(page.getByText('Campaign validated successfully.')).not.toBeVisible();

    // Pipeline step 1 should still be green (ingested)
    const step1 = pipelineStepCircle(page, 1);
    await expect(step1).toHaveAttribute('style', GREEN_STYLE);

    // Pipeline step 2 should still be gray (validation failed)
    const step2 = pipelineStepCircle(page, 2);
    await expect(step2).toHaveAttribute('style', GRAY_STYLE);
  });

});
