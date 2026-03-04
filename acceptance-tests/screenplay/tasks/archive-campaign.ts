import type { Performable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { Click } from '../interactions/click';
import { WaitForUrl } from '../interactions/wait-for-url';

export class ArchiveCampaign implements Performable {
  private constructor() {}

  static now(): ArchiveCampaign {
    return new ArchiveCampaign();
  }

  async performAs(actor: Actor): Promise<void> {
    await actor.attemptsTo(
      Click.theButton('Archive'),
      Click.theButton('Confirm Archive'),
      WaitForUrl.matching('/campaigns'),
    );
  }
}
