import { AfterViewInit, Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges } from '@angular/core';
import * as L from 'leaflet';
import { GeoSearchControl, OpenStreetMapProvider } from 'leaflet-geosearch';
import { LatLngExpression } from 'leaflet';
import { Category } from "../../data/Category";
import * as Icons from './icons';
import { Sight } from 'src/app/data/Sight';
import { RouteResponse } from 'src/app/services/route.service';
import { GPSService } from 'src/app/services/gps.service';

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

  routeSightLayer: L.LayerGroup;
  routeLayer: L.LayerGroup;
  routePoly?: L.Polyline;

  constructor(private gpsService: GPSService) {
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

    if (this.startPoint) {
      this.marker = new L.Marker(this.startPoint);
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
      center: [this.initLat!, this.initLng!],
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
      const icon = this.getIcon(sight);

      let newMarker = new L.Marker(latlng, { icon: icon, }).addTo(newLayer);
      newMarker.bindPopup(sight.category, { closeButton: false });
      newLayer.addTo(this.map);
    });
    this.activeLayers.set(category.name, newLayer);
  }

  hideSights(category: Category) {
    if (this.activeLayers.has(category.name)) {
      this.map.removeLayer(this.activeLayers.get(category.name));
    }
  }

  getIcon(sight: Sight) {
    const cat = sight.category;
    switch (cat) {
      case "Sightseeing":
        return Icons.sightsIcon;
      case "Nightlife":
        return Icons.nightIcon;
      case "Restaurants":
        return Icons.restaurantIcon;
      case "Shopping":
        return Icons.shoppingIcon;
      case "PicnicBarbequeSpot":
        return Icons.grillIcon;
      case "MuseumExhibition":
        return Icons.museumIcon;
      case "Nature":
        return Icons.natureIcon;
      case "Swimming":
        return Icons.seaIcon;
      default:
        return iconDefault;
    }
  }
  drawRoute(_route: RouteResponse) {
    this.hideRoute()
    this.routeLayer = new L.LayerGroup<any>();
    var r = 55;
    var g = 255;
    var colorStepsize = (g - r) / _route.route!.length;
    const _sections: L.LatLng[][] = [] = [];
    _route.route!.map(section => {
      _sections.push(section.nodes.map(node => new L.LatLng(node.lat, node.lon)));
      var sectionNodes: L.LatLng[] = [];
      section.nodes.map(node => {
        sectionNodes.push(new L.LatLng(node.lat, node.lon));
      });
      this.routePoly = new L.Polyline(sectionNodes, { color: "rgb(" + r + " ," + g + ",0)", weight: 6 }).addTo(this.routeLayer);
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
        const icon = this.getIcon(section.sight);
        var newMarker = new L.Marker(latlng, { icon: icon }).addTo(this.routeSightLayer);
        newMarker.bindPopup(section.sight.category, { closeButton: false });
        this.routeSightLayer.addTo(this.map);
      }
    });
  }

}
