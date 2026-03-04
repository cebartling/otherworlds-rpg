import type { Answerable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

export class TheButtonState implements Answerable<boolean> {
  private constructor(private readonly buttonName: string) {}

  static forButton(name: string): TheButtonState {
    return new TheButtonState(name);
  }

  async answeredBy(actor: Actor): Promise<boolean> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    return page.getByRole('button', { name: this.buttonName }).isEnabled();
  }
}
