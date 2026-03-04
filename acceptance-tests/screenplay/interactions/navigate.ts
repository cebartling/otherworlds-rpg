import type { Performable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

export class Navigate implements Performable {
  private constructor(private readonly path: string) {}

  static to(path: string): Navigate {
    return new Navigate(path);
  }

  async performAs(actor: Actor): Promise<void> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    await page.goto(this.path, { waitUntil: 'networkidle' });
  }
}
