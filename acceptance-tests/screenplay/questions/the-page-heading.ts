import type { Answerable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

export class ThePageHeading implements Answerable<string> {
  private constructor() {}

  static text(): ThePageHeading {
    return new ThePageHeading();
  }

  async answeredBy(actor: Actor): Promise<string> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    const heading = page.locator('h1');
    return (await heading.textContent()) ?? '';
  }
}
