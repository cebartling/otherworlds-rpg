import type { Answerable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

export class TheCampaignIdFromUrl implements Answerable<string> {
  private constructor() {}

  static value(): TheCampaignIdFromUrl {
    return new TheCampaignIdFromUrl();
  }

  async answeredBy(actor: Actor): Promise<string> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    const url = page.url();
    return url.split('/campaigns/')[1];
  }
}
