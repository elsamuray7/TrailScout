import { Sight } from './Sight';

export class Category {
  name: string;
  pref: number = 0;
  sights: Sight[] = [];

  public constructor(name: string) {
    this.name = name;
  }
}
