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

  radius?: number;
  constructor(private cookieService: CookieHandlerService) { }

  ngOnInit(): void {
    const cookies = this.cookieService.getPriosCookies();
    console.log(cookies);
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
