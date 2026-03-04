import type { Page } from '@playwright/test';
import type { Ability } from './actor';

export class BrowseTheWeb implements Ability {
  static readonly abilityName = 'BrowseTheWeb';
  readonly abilityName = BrowseTheWeb.abilityName;

  readonly page: Page;

  private constructor(page: Page) {
    this.page = page;
  }

  static using(page: Page): BrowseTheWeb {
    return new BrowseTheWeb(page);
  }
}
