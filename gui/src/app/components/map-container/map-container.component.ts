import { AfterViewInit, Component, EventEmitter, Input, OnChanges, Output, SimpleChanges } from '@angular/core';
import * as L from 'leaflet';
import { GeoSearchControl, OpenStreetMapProvider } from 'leaflet-geosearch';
import { LatLngExpression } from 'leaflet';
import {Category} from "../../data/Category";
import { RouteResponse, RouteService } from 'src/app/services/route.service';

const iconRetinaUrl = 'assets/marker-icon-2x.png';
const iconUrl = 'assets/marker-icon.png';
const shadowUrl = 'assets/marker-shadow.png';
const iconDefault = L.icon({
  iconRetinaUrl,
  iconUrl,
  shadowUrl,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  tooltipAnchor: [16, -28],
  shadowSize: [41, 41]
});
L.Marker.prototype.options.icon = iconDefault;

@Component({
  selector: 'app-map-container',
  templateUrl: './map-container.component.html',
  styleUrls: ['./map-container.component.scss']
})
export class MapContainerComponent implements AfterViewInit, OnChanges {
  map!: L.Map;

  @Input() initLat = 48.7758459;
  @Input() initLng = 9.1829321;
  @Input() initZoom = 10;
  @Input() circleRadius?: number;
  @Input() startPoint?: L.LatLng;

  @Output() markerLocation = new EventEmitter;
  private marker?: L.Marker;
  private circle?: L.Circle;
  private activeLayers = new Map<string, any>();

  leafRoute_1?: L.Polyline;
  leafRoute_2?: L.Polyline

  constructor(private routeService : RouteService) {
  }
  ngOnChanges(changes: SimpleChanges): void {
    this.addCircle(this.marker?.getLatLng()!);
  }

  ngAfterViewInit(): void {
    this.loadMap();
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
          resultFormat: function(t:any) {
            return "" + t.result.label;
          },
          updateMap: !0
    });
    this.map.addControl(searchControl);

    if (this.startPoint) {
      this.marker = new L.Marker(this.startPoint);
      this.marker.addTo(this.map);
      this.addCircle(this.startPoint);
      this.markerLocation.emit(this.startPoint)
    }
  }

  loadMap() {
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
  }

  async onClick(event: any, map: L.Map) {
    const latlng = event.latlng as L.LatLng;
    if (this.marker) {
      this.marker.removeFrom(map);
    }
    this.marker = new L.Marker(latlng);
    this.marker.addTo(map);
    this.addCircle(latlng);
    this.markerLocation.emit(latlng)
  }

  addCircle(latlng: L.LatLng) {
    if (this.circleRadius || this.circleRadius === 0) {
      if (this.circle) {
        this.circle.removeFrom(this.map);
      }
      if (latlng && this.circleRadius > 0) {
        this.circle = L.circle(latlng, this.circleRadius * 1000).addTo(this.map);
      }
    }
  }

  drawSights(category: Category) {
    var newLayer = new L.LayerGroup<any>();
    category.sights.forEach((sight) => {
      var latlng: LatLngExpression = {
        lat: sight.lat,
        lng: sight.lon
      }
      var newMarker = new L.Marker(latlng).addTo(newLayer);
      newLayer.addTo(this.map);
    });
    this.activeLayers.set(category.name, newLayer);
  }

  hideSights(category: Category) {
    if (this.activeLayers.has(category.name)) {
      this.map.removeLayer(this.activeLayers.get(category.name));
    }
  }

  drawRoute(_route: RouteResponse) {
    const route_1: L.LatLng[] = [];
    const route_2: L.LatLng[] = [];

    _route.route.map(nodes => nodes.nodes.map(node => {
      if (nodes.type === 'Start') {
        route_1.push(new L.LatLng(node.lat, node.lon))
        return;
      }
      if (nodes.type === 'End') {
        route_2.push(new L.LatLng(node.lat, node.lon))
      }
      
    }) );
    this.leafRoute_1 = new L.Polyline(route_1, {color: 'red'}).addTo(this.map);
    this.leafRoute_2 = new L.Polyline(route_2, {color: 'green'}).addTo(this.map);
  }
}
