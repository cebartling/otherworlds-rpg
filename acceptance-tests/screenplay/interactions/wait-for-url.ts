import type { Performable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

export class WaitForUrl implements Performable {
  private constructor(private readonly pattern: string | RegExp) {}

  static matching(pattern: string | RegExp): WaitForUrl {
    return new WaitForUrl(pattern);
  }

  async performAs(actor: Actor): Promise<void> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    await page.waitForURL(this.pattern);
  }
}
