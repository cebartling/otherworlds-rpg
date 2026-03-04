import { expect } from '@playwright/test';
import type { Performable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';
import { Navigate } from '../interactions/navigate';
import { Click } from '../interactions/click';
import { UploadFile } from '../interactions/upload-file';
import { WaitForUrl } from '../interactions/wait-for-url';

export class IngestCampaign implements Performable {
  private constructor(private readonly source: string) {}

  static withSource(source: string): IngestCampaign {
    return new IngestCampaign(source);
  }

  async performAs(actor: Actor): Promise<void> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);

    await actor.attemptsTo(
      Navigate.to('/campaigns'),
      Click.theButton('Ingest Campaign'),
    );

    // Wait for the toggle to reveal the file input (confirms JS hydration)
    await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible({
      timeout: 10_000,
    });

    await actor.attemptsTo(
      UploadFile.toInput('#campaign-file', {
        name: 'campaign.md',
        mimeType: 'text/markdown',
        content: this.source,
      }),
      Click.theButton('Ingest', { exact: true }),
      WaitForUrl.matching(/\/campaigns\/[0-9a-f-]{36}$/),
    );
  }
}
