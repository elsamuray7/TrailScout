import { Component, OnInit } from '@angular/core';
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
  sightsWithPrio?: SightsPrios;

  radius?: number;
  constructor(private cookieService: CookieHandlerService) { 
    const prioCookies = this.cookieService.getPriosCookies();
    if (prioCookies) {
      this.sightsWithPrio = {sightWithPrio: new Map<string, number>()};
      for (const cookie of prioCookies) {
        this.sightsWithPrio.sightWithPrio.set(cookie.key, cookie.value)
      }
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
  }

  markerSet(latlng: L.LatLng) {
    this.marker = true;
  }

}
