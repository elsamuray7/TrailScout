import { Injectable } from '@angular/core';
import {CookieHandlerService} from "./cookie-handler.service";

@Injectable({
  providedIn: 'root'
})
export class MapService {

  private coords: L.LatLng;
  private radius: number = 0;
  constructor(private cookieService: CookieHandlerService) {
    if (this.cookieService.getLocationCookie().value !== '') {
      const val = this.cookieService.getLocationCookie().value as string;
      this.coords = JSON.parse(val);
    }
    if (this.cookieService.getRadiusCookie().value !== '') {
      this.radius = parseInt(this.cookieService.getRadiusCookie().value);
    }
  }

  public setCoordinates(coords: L.LatLng) {
    this.coords = coords;
    this.cookieService.setLocationCookie(this.coords)
  }

  public setRadius(radius: number) {
    this.radius = radius;
    this.cookieService.setRadiusCookie(this.radius)
  }

  public getCoordniates() {
    return this.coords;
  }

  public getRadius() {
    return this.radius;
  }
}
