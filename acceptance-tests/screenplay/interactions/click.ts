import type { Performable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

export class Click implements Performable {
  private constructor(
    private readonly name: string,
    private readonly exact: boolean,
  ) {}

  static theButton(name: string, options?: { exact?: boolean }): Click {
    return new Click(name, options?.exact ?? false);
  }

  async performAs(actor: Actor): Promise<void> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    await page.getByRole('button', { name: this.name, exact: this.exact }).click();
  }
}
