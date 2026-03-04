import type { Performable, Answerable } from './interfaces';

export interface Ability {
  abilityName: string;
}

export class Actor {
  private readonly name: string;
  private readonly abilities = new Map<string, Ability>();

  private constructor(name: string) {
    this.name = name;
  }

  static named(name: string): Actor {
    return new Actor(name);
  }

  whoCan(...abilities: Ability[]): this {
    for (const ability of abilities) {
      this.abilities.set(ability.abilityName, ability);
    }
    return this;
  }

  abilityTo<T extends Ability>(abilityClass: { abilityName: string }): T {
    const ability = this.abilities.get(abilityClass.abilityName);
    if (!ability) {
      throw new Error(
        `${this.name} does not have the ability: ${abilityClass.abilityName}`,
      );
    }
    return ability as T;
  }

  async attemptsTo(...activities: Performable[]): Promise<void> {
    for (const activity of activities) {
      await activity.performAs(this);
    }
  }

  async asks<T>(question: Answerable<T>): Promise<T> {
    return question.answeredBy(this);
  }
}
