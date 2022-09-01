import { Injectable } from '@angular/core';
import {CookieHandlerService} from "./cookie-handler.service";
import {SightsServiceService} from "./sights-service.service";

@Injectable({
  providedIn: 'root'
})
export class MapService {

  private coords: L.LatLng;
  private radius: number = 0;
  private timeout: any;
  constructor(private cookieService: CookieHandlerService,
              private sightsService: SightsServiceService) {
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
    this.cookieService.setLocationCookie(this.coords);
    this.setAutoRefresh();
  }

  public setRadius(radius: number) {
    this.radius = radius;
    this.cookieService.setRadiusCookie(this.radius);
    this.setAutoRefresh();
  }

  private setAutoRefresh() {
    if (this.radius > 0 && this.coords) {
      clearTimeout(this.timeout)
      this.timeout = setTimeout(() => this.sightsService.updateSights(this.getCoordniates(), this.getRadius()), 500);
    }
  }

  public getCoordniates(): L.LatLng {
    return this.coords;
  }

  public getRadius(): number {
    return this.radius;
  }
}
