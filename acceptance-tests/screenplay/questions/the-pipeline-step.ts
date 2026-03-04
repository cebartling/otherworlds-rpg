import type { Locator } from '@playwright/test';
import type { Answerable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

/** RegExp matching the green background style (hex from SSR, rgb from client-side). */
export const PIPELINE_STEP_GREEN = /background-color: (#2e7d32|rgb\(46, 125, 50\))/;

/** RegExp matching the gray border style on inactive pipeline steps/badges. */
export const PIPELINE_STEP_GRAY = /border: 1px solid var\(--color-border\)/;

export class ThePipelineStep implements Answerable<Locator> {
  private constructor(private readonly stepNumber: number) {}

  static circle(stepNumber: number): ThePipelineStep {
    return new ThePipelineStep(stepNumber);
  }

  async answeredBy(actor: Actor): Promise<Locator> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    return page
      .locator('section')
      .filter({ hasText: 'Content Pipeline' })
      .locator('.rounded-full')
      .filter({ hasText: String(this.stepNumber) })
      .first();
  }
}
