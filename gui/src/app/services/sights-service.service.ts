import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class SightsServiceService {

  constructor() { }

  public getSights(coords: L.LatLng, radius: number) {
    const lat = coords["lat"];
    const lon = coords["lng"];
    console.log()
  }
}
