import { AfterViewInit, Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges } from '@angular/core';
import * as L from 'leaflet';

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

  @Output() markerLocation = new EventEmitter;
  private marker?: L.Marker;
  private circle?: L.Circle;

  constructor() { 
  }
  ngOnChanges(changes: SimpleChanges): void {
    this.circleRadius = changes["circleRadius"].currentValue;
    this.addCircle(this.marker?.getLatLng()!);
  }

  ngAfterViewInit(): void {
    this.loadMap();
    this.map.on('click', event => this.onClick(event, this.map));
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
    
  }

  addCircle(latlng: L.LatLng) {
    if (this.circleRadius) {
      if (this.circle) {
        this.circle.removeFrom(this.map);
      }
      this.circle = L.circle(latlng, this.circleRadius * 1000).addTo(this.map);
    }
  }

}
