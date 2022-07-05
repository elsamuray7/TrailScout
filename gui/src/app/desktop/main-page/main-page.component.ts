import { Component, OnInit } from '@angular/core';
import { NgbOffcanvas } from '@ng-bootstrap/ng-bootstrap';
import * as L from 'leaflet';
import { CookieHandlerService } from 'src/app/services/cookie-handler.service';
import { Settings, Sight, SightsPrios } from 'src/app/types.utils';

@Component({
  selector: 'app-main-page',
  templateUrl: './main-page.component.html',
  styleUrls: ['./main-page.component.scss']
})
export class MainPageComponent implements OnInit {

  //CATEGORIES
  data = {
    "categories": [
        {
            "id": "0",
            "name": "Tiere",
            "pref": 0,
            "image": "animals.jpg"
        },
        {
            "id": "1",
            "name": "Gastronomie",
            "pref": 0,
            "image": "gastro.jpg"
        },
        {
            "id": "2",
            "name": "Aktivitäten",
            "pref": 0,
            "image": "activity.jpg"
        },
        {
            "id": "3",
            "name": "Sehenswürdigkeiten",
            "pref": 0,
            "image": "sights.jpg"
        },
        {
            "id": "4",
            "name": "Nachtleben",
            "pref": 0,
            "image": "nightlife.jpg"
        },
        {
            "id": "5",
            "name": "Aussichtspunkte",
            "pref": 0,
            "image": "viewpoint.jpg"
        },
        {
            "id": "6",
            "name": "Shops",
            "pref": 0,
            "image": "shops.jpg"
        },
        {
            "id": "7",
            "name": "Grill-/Picknickplätze",
            "pref": 0,
            "image": "grill.jpg"
        },
        {
            "id": "8",
            "name": "Baden und Seen",
            "pref": 0,
            "image": "see.jpg"
        },
        {
            "id": "9",
            "name": "Kunst und Kultur",
            "pref": 0,
            "image": "art.jpg"
        }
    ]
}

  sights: Sight[] = [];

  marker = false;
  markerCoords?: L.LatLng;
  sightsWithPrio?: SightsPrios;
  isCollapsed = true;

  radius?: number;
  constructor(
    private cookieService: CookieHandlerService, 
    private offcanvasService: NgbOffcanvas) { 

    this.sights = this.mapData();

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
    if (!this.radius) {
      this.radius = 0;
    }
    this.cookieService.setRadiusCookie(this.radius);
  }

  markerSet(latlng: L.LatLng) {
    this.marker = true;
    this.markerCoords = latlng;
    this.cookieService.setLocationCookie(latlng);
  }

  collapse() {
    this.isCollapsed = !this.isCollapsed;
  }

  open(content: any) {
    if (!this.markerCoords) {
      return;
    }
    this.offcanvasService.open(content).result.then((result) => {
      console.log(result);
    }, (reason) => {
      console.log(reason);
    })
  }

  mapData() {
    return this.data.categories.map(s => {
      return <Sight> {
        id: s.id,
        name: s.name,
        description: s.name,
        pref: s.pref,
        imagePath: s.image
      }
    })
  }
}
