import { AfterViewInit, Component, Input, OnInit } from '@angular/core';
import * as L from 'leaflet';

@Component({
  selector: 'app-map-container',
  templateUrl: './map-container.component.html',
  styleUrls: ['./map-container.component.scss']
})
export class MapContainerComponent implements AfterViewInit {
  map!: L.Map;

  @Input() initLat = 48.7758459;
  @Input() initLng = 9.1829321;
  @Input() initZoom = 10;

  constructor() { 
  }

  ngAfterViewInit(): void {
    this.loadMap();
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

}
