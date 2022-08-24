import { Component, Input, OnInit } from '@angular/core';

@Component({
  selector: 'app-route-tracker',
  templateUrl: './route-tracker.component.html',
  styleUrls: ['./route-tracker.component.scss']
})
export class RouteTrackerComponent implements OnInit {

  @Input() sections!: L.LatLng[][];

  constructor() { }

  ngOnInit(): void {
  }

}
