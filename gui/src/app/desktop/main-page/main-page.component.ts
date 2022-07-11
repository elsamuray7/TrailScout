import { Component, OnInit, ViewChild } from '@angular/core';
import { NgbOffcanvas } from '@ng-bootstrap/ng-bootstrap';
import * as L from 'leaflet';
import { CookieHandlerService } from 'src/app/services/cookie-handler.service';
import { MapContainerComponent } from '../../components/map-container/map-container.component';

@Component({
  selector: 'app-main-page',
  templateUrl: './main-page.component.html',
  styleUrls: ['./main-page.component.scss']
})
export class MainPageComponent implements OnInit {

  @ViewChild(MapContainerComponent) mapContainer: MapContainerComponent;
  marker = false;
  markerCoords?: L.LatLng;
  isCollapsed = true;

  defaultStartPointLong = 8.806422;
  defaultStartPointLat = 53.073635;

  radius?: number;
  constructor(
    private cookieService: CookieHandlerService,
    private offcanvasService: NgbOffcanvas) {

    const startCookie = this.cookieService.getLocationCookie();
    if (startCookie.value !== '') {
      const val = startCookie.value as string;
      const coords = JSON.parse(val);
      this.markerSet(new L.LatLng(coords["lat"] as any, coords["lng"] as any))
    }

    const radiusCookie = this.cookieService.getRadiusCookie();
    if (radiusCookie && !this.radius) {
      this.radiusChange(radiusCookie.value as number);
    }
  }

  ngOnInit(): void {

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

  drawSights(response: any) {
    if (response.drawSight) {
      this.mapContainer.drawSights(response.category);
    } else {
      this.mapContainer.hideSights(response.category);
    }
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
}
