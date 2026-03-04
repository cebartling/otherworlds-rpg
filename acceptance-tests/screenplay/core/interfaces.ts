import type { Actor } from './actor';

export interface Performable {
  performAs(actor: Actor): Promise<void>;
}

export interface Answerable<T> {
  answeredBy(actor: Actor): Promise<T>;
}
