import { Sight } from './Sight';

export class Category {
  name: string;
  preference: number = 3;
  sights: Sight[] = [];

  public constructor(name: string) {
    this.name = name;
  }
}
