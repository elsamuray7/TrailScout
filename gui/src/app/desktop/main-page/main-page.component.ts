import { Component, OnInit } from '@angular/core';
import * as L from 'leaflet';
import { CookieHandlerService } from 'src/app/services/cookie-handler.service';
import { Settings, Sight, SightsPrios } from 'src/app/types.utils';

@Component({
  selector: 'app-main-page',
  templateUrl: './main-page.component.html',
  styleUrls: ['./main-page.component.scss']
})
export class MainPageComponent implements OnInit {

  //TEST DATA
  sights: Sight[] = [
    {name: 'Aussichtspunkt', id : '1'},
    {name: 'Baum', id : '2'},
    {name: 'Statue', id : '3'},
    {name: 'Park', id : '4'},
    {name: 'Restaurant', id : '5'}
  ]

  marker = false;
  markerCoords?: L.LatLng;
  sightsWithPrio?: SightsPrios;
  isCollapsed = false;

  radius?: number;
  constructor(private cookieService: CookieHandlerService) { 
    const prioCookies = this.cookieService.getPriosCookies();
    if (prioCookies) {
      this.sightsWithPrio = {sightWithPrio: new Map<string, number>()};
      for (const cookie of prioCookies) {
        this.sightsWithPrio.sightWithPrio.set(cookie.key, cookie.value)
      }
    }
    const startCookie = this.cookieService.getLocationCookie();
    if (startCookie.value !== '') {
      const val = startCookie.value as string;
      const coords = val.substring(val.indexOf('(') + 1, val.indexOf(')')).split(',');
      this.markerSet(new L.LatLng(coords[0] as any, coords[1] as any))
    }

    const radiusCookie = this.cookieService.getRadiusCookie();
    if (radiusCookie && !this.radius) {
      this.radiusChange(radiusCookie.value as number);
    }
  }

  ngOnInit(): void {
    
  }

  getSettings(result: Settings) {
    const sightsWithPrios: SightsPrios = {sightWithPrio: result.sights};
    this.cookieService.setPriosCookie(sightsWithPrios)
  }

  radiusChange(radius: number) {
    this.radius = radius;
    this.cookieService.setRadiusCookie(radius);
  }

  markerSet(latlng: L.LatLng) {
    this.marker = true;
    this.markerCoords = latlng;
    this.cookieService.setLocationCookie(latlng);
  }

}
