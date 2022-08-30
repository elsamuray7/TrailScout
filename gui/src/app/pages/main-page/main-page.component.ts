import { Component, OnDestroy, OnInit, ViewChild } from '@angular/core';
import { NgbOffcanvas } from '@ng-bootstrap/ng-bootstrap';
import * as L from 'leaflet';
import { BlockUI, NgBlockUI } from 'ng-block-ui';
import { Subscription } from 'rxjs';
import { ApplicationStateService } from 'src/app/services/application-state.service';
import { GPSService } from 'src/app/services/gps.service';
import { RouteResponse, RouteService } from 'src/app/services/route.service';
import { ToastService } from 'src/app/services/toast.service';
import { RouteTrackerSection } from 'src/app/types.utils';
import { MapContainerComponent } from '../../components/map-container/map-container.component';
import {MapService} from "../../services/map.service";

@Component({
  selector: 'app-main-page',
  templateUrl: './main-page.component.html',
  styleUrls: ['./main-page.component.scss']
})
export class MainPageComponent implements OnInit, OnDestroy {

  @ViewChild(MapContainerComponent) mapContainer: MapContainerComponent;
  @BlockUI('map') blockUIMap: NgBlockUI;
  marker = false;
  markerCoords?: L.LatLng;
  isCollapsed = true;

  mobile = false;

  sub?: Subscription;
  blockSub?: Subscription;
  routeTrackerSections: RouteTrackerSection[] = [];

  gpsPosition?: L.LatLng;

  radius?: number;
  constructor(
    private mapService: MapService,
    private offcanvasService: NgbOffcanvas,
    private applicationStateService: ApplicationStateService,
    private routeService: RouteService,
    private toastService: ToastService,
    private gpsService: GPSService) {

    this.mobile =  applicationStateService.getIsMobileResolution();

    this.sub = this.routeService.routeUpdated.subscribe(route => {
      this.showRoute(route);
      this.blockUIMap.stop();
    });
    this.blockSub = this.routeService.startRouteCall.subscribe(() => {
      this.blockUIMap.start('Loading route...');
    });

   

    const gps = this.gpsService.getLocation().subscribe(pos => {
      this.gpsPosition = pos as L.LatLng;
    });


    const coords = this.mapService.getCoordniates();
    if (coords) {
      this.markerSet(new L.LatLng(coords["lat"] as any, coords["lng"] as any))
    }

    const radius = this.mapService.getRadius();
    if (radius && !this.radius) {
      this.radiusChange(radius);
    }
  }
  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  async ngOnInit() {
    this.gpsPosition = await this.gpsService.getCurrentLocation();
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

  async showRoute(route: RouteResponse) {
    if (route.error && !route.route) {
      this.toastService.showDanger(route.error.message ?? 'Something went wrong!');
      return;
    }
    route.route?.map((r,index) => r.id = index);
    this.mapContainer.drawRoute(route);
    this.mapContainer.drawSightsOnRoute(route);
    this.routeTrackerSections = [];
    route.route?.forEach(route => {
      const sight = route.sight;
      const section = route.nodes.map(l => new L.LatLng(l.lat, l.lon));
      this.routeTrackerSections.push({section: section, sight: sight!, routeId: route.id!});
    });
    
  }
}
