import { Component, OnInit, ViewChild } from '@angular/core';
import { NgbOffcanvas } from '@ng-bootstrap/ng-bootstrap';
import * as L from 'leaflet';
import { ApplicationStateService } from 'src/app/services/application-state.service';
import { MapContainerComponent } from '../../components/map-container/map-container.component';
import {MapService} from "../../services/map.service";

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

  mobile = false;

  defaultStartPointLong = 8.806422;
  defaultStartPointLat = 53.073635;

  radius?: number;
  constructor(
    private mapService: MapService,
    private offcanvasService: NgbOffcanvas,
    private applicationStateService: ApplicationStateService) {

    this.mobile =  applicationStateService.getIsMobileResolution();

    const coords = this.mapService.getCoordniates();
    if (coords) {
      this.markerSet(new L.LatLng(coords["lat"] as any, coords["lng"] as any))
    }

    const radius = this.mapService.getRadius();
    if (radius && !this.radius) {
      this.radiusChange(radius);
    }
  }

  ngOnInit(): void {

  }

  radiusChange(radius: number) {
    this.radius = radius;
    if (!this.radius) {
      this.radius = 0;
    }
    this.mapService.setRadius(radius);
  }

  markerSet(latlng: L.LatLng) {
    this.marker = true;
    this.markerCoords = latlng;
    this.mapService.setCoordinates(latlng);
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
