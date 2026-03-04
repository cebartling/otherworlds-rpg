import { expect } from '@playwright/test';
import type { Performable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';
import { Click } from '../interactions/click';

export class CompileCampaign implements Performable {
  private constructor() {}

  static now(): CompileCampaign {
    return new CompileCampaign();
  }

  async performAs(actor: Actor): Promise<void> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);

    await actor.attemptsTo(Click.theButton('Compile'));

    await expect(page.getByText('Campaign compiled successfully.')).toBeVisible();
  }
}
