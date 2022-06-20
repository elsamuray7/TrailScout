import { Injectable } from '@angular/core';
import { environment } from 'src/environments/environment';
import { Sight } from '../types.utils';
import sightsJson from '../../../sights.json'

@Injectable({
  providedIn: 'root'
})
export class JsonHandlerService {

  constructor() { }

  getSights(): Sight[] {
    const sights: Sight[] = sightsJson.categories.map(s => {
      return <Sight> {
        id: s.id,
        name: s.name,
        description: s.name,
        pref: s.pref,
        imagePath: s.image
      }
    });

    return sights;
  }
}
