import { AfterViewInit, Component, EventEmitter, Input, OnChanges, OnDestroy, OnInit, Output, SimpleChanges } from '@angular/core';
import * as L from 'leaflet';
import { GeoSearchControl, OpenStreetMapProvider } from 'leaflet-geosearch';
import { LatLngExpression } from 'leaflet';
import { Category } from "../../data/Category";
import * as Icons from '../icons';
import { Sight } from 'src/app/data/Sight';
import { RouteResponse, RouteService } from 'src/app/services/route.service';
import { GPSService } from 'src/app/services/gps.service';
import { Subscription } from 'rxjs';
import { ApplicationStateService } from '../../services/application-state.service';


L.Marker.prototype.options.icon = Icons.iconDefault;

@Component({
  selector: 'app-map-container',
  templateUrl: './map-container.component.html',
  styleUrls: ['./map-container.component.scss']
})
export class MapContainerComponent implements AfterViewInit, OnChanges, OnDestroy {
  map!: L.Map;

  @Input() initLat?: number;
  @Input() initLng?: number;
  @Input() initZoom = 10;
  @Input() circleRadius?: number;
  @Input() startPoint?: L.LatLng;

  @Output() markerLocation = new EventEmitter;
  @Output('sections') _sectionEvent = new EventEmitter;
  private marker?: L.Marker;
  private circle?: L.Circle;
  private activeLayers = new Map<string, any>();

  currentLocation: L.Marker | undefined;

  routeSightLayer: L.LayerGroup;
  routeLayer: L.LayerGroup;
  routePoly: L.Polyline[] = [];

  sub1?: Subscription;
  sub2?: Subscription;
  sub3?: Subscription;

  constructor(private applicationStateService: ApplicationStateService,
              private gpsService: GPSService, private routeService: RouteService) {
    this.applicationStateService.routeModeChangedEvent.subscribe(isActive => {
      if (isActive) {
        //hide Start point, Radius and Settings
        //this.hideMarker();
        this.hideCircle();
        this.hideAllSights();
        const route = this.routeService.getRoute();
        if (route != null) {
          this.drawRoute(route);
          this.drawSightsOnRoute(route);
        }
      } else {
        //this.showStartPoint();
        if(this.startPoint) {
          this.addCircle(this.startPoint);
        }
        this.showAllActiveSights();
        this.hideRoute();
        this.hideSightsOnRoute();
      }
    })
  }
  ngOnDestroy() {
    this.sub1?.unsubscribe();
    this.sub2?.unsubscribe();
    this.sub3?.unsubscribe();
  }

  ngOnChanges(changes: SimpleChanges): void {
    this.addCircle(this.marker?.getLatLng()!);
  }

  async ngAfterViewInit() {
    await this.loadMap();
    this.map.on('click', event => this.onClick(event, this.map));

    const searchControl = GeoSearchControl({
      provider: new OpenStreetMapProvider(),
      style: 'bar',
      position: 'topleft',
      showMarker: false,
      marker: {
        draggable: true,
      },
      maxMarker: 1,
      autoClose: true,
      autoComplete: true,
      retainZoomLevel: true,
      maxSuggestions: 5,
      keepResult: true,
      resultFormat: function (t: any) {
        return "" + t.result.label;
      },
      updateMap: !0
    });
    this.map.addControl(searchControl);

    this.showStartPoint();
    this.markerLocation.emit(this.startPoint);
    if (this.startPoint) {
      this.marker = new L.Marker(this.startPoint, {icon: Icons.startIcon});
      this.marker.addTo(this.map);
      this.addCircle(this.startPoint);
      this.markerLocation.emit(this.startPoint)
    }

  }

  async loadMap() {
    if (!this.initLat && !this.initLng) {
      this.initLat = (await this.gpsService.getCurrentLocation())?.lat;
      this.initLng = (await this.gpsService.getCurrentLocation())?.lng;
      if (!this.initLat && !this.initLng) {
        this.initLat = 48.783333;
        this.initLng = 9.183333;
      }
    }
    this.map = L.map('map', {
      center: [this.initLat, this.initLng],
      zoom: this.initZoom
    });

    const tiles = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      minZoom: 3,
      attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
    });

    tiles.addTo(this.map);

    this.sub1 = this.routeService.id$.subscribe(id => {
      this.highlightSection(id);
    });
    this.sub2 = this.routeService.id_clicked$.subscribe(id => {
      this.showSection(id);
    });
    this.sub3 = this.gpsService.getLocation().subscribe((location: any) => {
      this.currentLocation?.removeFrom(this.map);
      this.currentLocation = new L.Marker(location);
      this.currentLocation.addTo(this.map);
    })
  }

  async onClick(event: any, map: L.Map) {
    const latlng = event.latlng as L.LatLng;
    this.hideMarker();
    this.marker = new L.Marker(latlng);
    if (this.marker) {
      this.marker.removeFrom(map);
    }
    this.marker = new L.Marker(latlng, {icon: Icons.startIcon});
    this.marker.addTo(map);
    this.addCircle(latlng);
    this.markerLocation.emit(latlng)
  }

  showStartPoint() {
    if (this.startPoint) {
      this.marker = new L.Marker(this.startPoint);
      this.marker.addTo(this.map);
      this.addCircle(this.startPoint);
    }
  }

  addCircle(latlng: L.LatLng) {
    if (this.circleRadius || this.circleRadius === 0) {
      this.hideCircle();
      if (latlng && this.circleRadius > 0) {
        this.circle = L.circle(latlng, this.circleRadius * 1000).addTo(this.map);
      }
    }
  }

  hideCircle() {
    if (this.circle) {
      this.circle.removeFrom(this.map);
    }
  }

  hideMarker() {
    if (this.marker) {
      this.marker.removeFrom(this.map);
    }
  }

  drawSights(category: Category) {
    var newLayer = new L.LayerGroup<any>();
    category.sights.forEach((sight) => {
      var latlng: LatLngExpression = {
        lat: sight.lat,
        lng: sight.lon
      }
      const icon = Icons.getIcon(sight);

      let newMarker = new L.Marker(latlng, {icon: icon,}).addTo(newLayer);
      newMarker.bindPopup(sight.name,{closeButton: false});
      newLayer.addTo(this.map);
    });
    this.activeLayers.set(category.name, newLayer);
  }

  hideSights(category: Category) {
    if (this.activeLayers.has(category.name)) {
      this.map.removeLayer(this.activeLayers.get(category.name));
      this.activeLayers.delete(category.name);
    }
  }

  hideAllSights() {
    for (let category of this.activeLayers.values()) {
      this.map.removeLayer(category);
    }
  }

  showAllActiveSights() {
    for (let category of this.activeLayers.values()) {
      this.map.addLayer(category);
    }
  }


  drawRoute(_route: RouteResponse) {
    this.hideRoute();
    this.routeLayer = new L.LayerGroup<any>();
    this.routePoly = [];
    var r = 55;
    var g = 255;
    var colorStepsize = (g - r) / _route.route!.length;
    const _sections: L.LatLng[][] = [] = [];
    _route.route!.forEach(section => {
      _sections.push(section.nodes.map(node => new L.LatLng(node.lat, node.lon)));
      var sectionNodes: L.LatLng[] = [];
      section.nodes.forEach(node => {
        sectionNodes.push(new L.LatLng(node.lat, node.lon));
      });
      this.routePoly.push(new L.Polyline(sectionNodes, { color: "rgb(" + r + " ," + g + ",0)", weight: 6 }).addTo(this.routeLayer));
        r += colorStepsize;
        g -= colorStepsize;

    });
    this.routeLayer.addTo(this.map);
    this._sectionEvent.emit(_sections);
  }

  hideRoute() {
    this.routeLayer?.removeFrom(this.map);
  }

  hideSightsOnRoute() {
    this.routeSightLayer?.removeFrom(this.map);
  }

  drawSightsOnRoute(route: RouteResponse) {
    this.hideSightsOnRoute();
    this.routeSightLayer = new L.LayerGroup<any>();
    route.route!.map(section => {
      if (section.sight) {
        var latlng: LatLngExpression = {
          lat: section.sight.lat,
          lng: section.sight.lon
        }
        const icon = Icons.getIcon(section.sight);
        var newMarker = new L.Marker(latlng, {icon: icon}).addTo(this.routeSightLayer);
        newMarker.bindPopup(section.sight.name,{closeButton: false});
        this.routeSightLayer.addTo(this.map);
      }
    });
  }

  highlightSection(id: number | null) {
    if (id === null) {
      this.routePoly.forEach(r => r.setStyle({weight: 6}));
      return;
    }
    const poly = this.routePoly.find((r,index) =>  index === id);
    poly?.setStyle({weight: 14});
  }

  showSection(id: number) {
    const poly = this.routePoly.find((r, index) => index === id);
    const bounds = poly?.getBounds();
    if (bounds) {
      this.map.fitBounds(bounds);
    }

  }

  showSight(sight: Sight) {
    this.map.flyTo(new L.LatLng(sight.lat, sight.lon), 19);
  }
}
