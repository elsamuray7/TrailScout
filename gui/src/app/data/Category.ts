import { Sight } from './Sight';

export class Category {
  name: string;
  pref: number = 0;
  sights: Sight[] = [];

  public constructor(name: string) {
    this.name = name;
  }

  public getAllSightsWithSpecialPref(): Sight[] {
    return this.sights.filter((sight) => sight.pref > -1);
  }
}
