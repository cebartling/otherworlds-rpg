import type { Performable } from '../core/interfaces';
import type { Actor } from '../core/actor';
import { BrowseTheWeb } from '../core/browse-the-web';

interface FileOptions {
  name: string;
  mimeType: string;
  content: string;
}

export class UploadFile implements Performable {
  private constructor(
    private readonly selector: string,
    private readonly fileOptions: FileOptions,
  ) {}

  static toInput(selector: string, options: FileOptions): UploadFile {
    return new UploadFile(selector, options);
  }

  async performAs(actor: Actor): Promise<void> {
    const { page } = actor.abilityTo<BrowseTheWeb>(BrowseTheWeb);
    await page.locator(this.selector).setInputFiles({
      name: this.fileOptions.name,
      mimeType: this.fileOptions.mimeType,
      buffer: Buffer.from(this.fileOptions.content, 'utf-8'),
    });
  }
}
