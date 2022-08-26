import { Component, Input, OnInit } from '@angular/core';

@Component({
  selector: 'app-route-tracker-item',
  templateUrl: './route-tracker-item.component.html',
  styleUrls: ['./route-tracker-item.component.scss']
})
export class RouteTrackerItemComponent implements OnInit {

  @Input() section: L.LatLng[];

  constructor() { }

  ngOnInit(): void {
  }

}
