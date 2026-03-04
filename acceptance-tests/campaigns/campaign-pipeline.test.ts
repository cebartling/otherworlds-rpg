import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  Actor,
  BrowseTheWeb,
  IngestCampaign,
  ValidateCampaign,
  CompileCampaign,
  ArchiveCampaign,
  Navigate,
  TheCampaignIdFromUrl,
  ThePageHeading,
  ThePipelineStep,
  TheButtonState,
  PIPELINE_STEP_GREEN,
  PIPELINE_STEP_GRAY,
} from '../screenplay';

const __dirname = dirname(fileURLToPath(import.meta.url));

const VALID_CAMPAIGN_SOURCE = readFileSync(
  resolve(__dirname, '../fixtures/the-lost-temple.md'),
  'utf-8',
);

const INVALID_CAMPAIGN_SOURCE = 'This has no YAML front-matter and no scenes.';

test.describe('Campaign Pipeline', () => {

  test('ingest campaign via file upload', async ({ page }) => {
    const actor = Actor.named('Campaign Author').whoCan(BrowseTheWeb.using(page));

    await actor.attemptsTo(IngestCampaign.withSource(VALID_CAMPAIGN_SOURCE));

    const campaignId = await actor.asks(TheCampaignIdFromUrl.value());

    // Should be on the detail page with a valid UUID
    expect(campaignId).toMatch(/^[0-9a-f-]{36}$/);

    // The heading should display the campaign ID prefix
    const headingText = await actor.asks(ThePageHeading.text());
    expect(headingText).toContain(campaignId.substring(0, 8));

    // Pipeline step 1 (Ingested) should be green
    const step1 = await actor.asks(ThePipelineStep.circle(1));
    await expect(step1).toHaveAttribute('style', PIPELINE_STEP_GREEN);

    // Pipeline steps 2 and 3 should be gray (not yet reached)
    const step2 = await actor.asks(ThePipelineStep.circle(2));
    await expect(step2).toHaveAttribute('style', PIPELINE_STEP_GRAY);
    const step3 = await actor.asks(ThePipelineStep.circle(3));
    await expect(step3).toHaveAttribute('style', PIPELINE_STEP_GRAY);

    // Version should show v1 in the Version Details section
    const versionSection = page.locator('section').filter({ hasText: 'Version Details' });
    await expect(versionSection.locator('p', { hasText: 'v1' })).toBeVisible();

    // Version hash should be present (not N/A)
    const hashText = await versionSection.locator('p.font-mono').textContent();
    expect(hashText).not.toBe('N/A');
    expect(hashText!.length).toBeGreaterThan(10);

    // Validate button should be enabled (ingested but not validated)
    expect(await actor.asks(TheButtonState.forButton('Validate'))).toBe(true);

    // Compile button should be disabled (not yet validated)
    expect(await actor.asks(TheButtonState.forButton('Compile'))).toBe(false);
  });

  test('validate ingested campaign', async ({ page }) => {
    const actor = Actor.named('Campaign Author').whoCan(BrowseTheWeb.using(page));

    await actor.attemptsTo(
      IngestCampaign.withSource(VALID_CAMPAIGN_SOURCE),
      ValidateCampaign.now(),
    );

    // Pipeline steps 1 and 2 should now be green
    const step1 = await actor.asks(ThePipelineStep.circle(1));
    await expect(step1).toHaveAttribute('style', PIPELINE_STEP_GREEN);
    const step2 = await actor.asks(ThePipelineStep.circle(2));
    await expect(step2).toHaveAttribute('style', PIPELINE_STEP_GREEN);

    // Pipeline step 3 should still be gray
    const step3 = await actor.asks(ThePipelineStep.circle(3));
    await expect(step3).toHaveAttribute('style', PIPELINE_STEP_GRAY);

    // Validate button should now be disabled (already validated)
    expect(await actor.asks(TheButtonState.forButton('Validate'))).toBe(false);

    // Compile button should now be enabled (validated but not compiled)
    expect(await actor.asks(TheButtonState.forButton('Compile'))).toBe(true);
  });

  test('compile validated campaign', async ({ page }) => {
    const actor = Actor.named('Campaign Author').whoCan(BrowseTheWeb.using(page));

    await actor.attemptsTo(
      IngestCampaign.withSource(VALID_CAMPAIGN_SOURCE),
      ValidateCampaign.now(),
      CompileCampaign.now(),
    );

    // All three pipeline steps should be green
    const step1 = await actor.asks(ThePipelineStep.circle(1));
    await expect(step1).toHaveAttribute('style', PIPELINE_STEP_GREEN);
    const step2 = await actor.asks(ThePipelineStep.circle(2));
    await expect(step2).toHaveAttribute('style', PIPELINE_STEP_GREEN);
    const step3 = await actor.asks(ThePipelineStep.circle(3));
    await expect(step3).toHaveAttribute('style', PIPELINE_STEP_GREEN);

    // Both buttons should be disabled
    expect(await actor.asks(TheButtonState.forButton('Validate'))).toBe(false);
    expect(await actor.asks(TheButtonState.forButton('Compile'))).toBe(false);
  });

  test('archive campaign removes from list', async ({ page }) => {
    const actor = Actor.named('Campaign Author').whoCan(BrowseTheWeb.using(page));

    await actor.attemptsTo(IngestCampaign.withSource(VALID_CAMPAIGN_SOURCE));

    const campaignId = await actor.asks(TheCampaignIdFromUrl.value());

    await actor.attemptsTo(ArchiveCampaign.now());

    // Campaign should no longer appear in the list
    await expect(page.locator(`a[href="/campaigns/${campaignId}"]`)).toHaveCount(0);
  });

  test('campaigns list shows ingested campaign with correct badge state', async ({ page }) => {
    const actor = Actor.named('Campaign Author').whoCan(BrowseTheWeb.using(page));

    await actor.attemptsTo(IngestCampaign.withSource(VALID_CAMPAIGN_SOURCE));

    const campaignId = await actor.asks(TheCampaignIdFromUrl.value());

    await actor.attemptsTo(Navigate.to('/campaigns'));

    // The specific campaign card should be present (link to its detail page)
    const campaignCard = page.locator(`a[href="/campaigns/${campaignId}"]`);
    await expect(campaignCard).toBeVisible();

    // The Ingested badge on this card should be green-styled
    const ingestedBadge = campaignCard.locator('span').filter({ hasText: 'Ingested' });
    await expect(ingestedBadge).toHaveAttribute('style', PIPELINE_STEP_GREEN);

    // The Validated badge should be gray (not green)
    const validatedBadge = campaignCard.locator('span').filter({ hasText: 'Validated' });
    await expect(validatedBadge).toHaveAttribute('style', PIPELINE_STEP_GRAY);

    // The Compiled badge should be gray (not green)
    const compiledBadge = campaignCard.locator('span').filter({ hasText: 'Compiled' });
    await expect(compiledBadge).toHaveAttribute('style', PIPELINE_STEP_GRAY);
  });

  test('validation error displays inline', async ({ page }) => {
    const actor = Actor.named('Campaign Author').whoCan(BrowseTheWeb.using(page));

    await actor.attemptsTo(IngestCampaign.withSource(INVALID_CAMPAIGN_SOURCE));

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
    const step1 = await actor.asks(ThePipelineStep.circle(1));
    await expect(step1).toHaveAttribute('style', PIPELINE_STEP_GREEN);

    // Pipeline step 2 should still be gray (validation failed)
    const step2 = await actor.asks(ThePipelineStep.circle(2));
    await expect(step2).toHaveAttribute('style', PIPELINE_STEP_GRAY);
  });

});
