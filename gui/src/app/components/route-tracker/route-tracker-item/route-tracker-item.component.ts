import { Component, Input, OnInit } from '@angular/core';
import { RouteTrackerSection } from 'src/app/types.utils';

@Component({
  selector: 'app-route-tracker-item',
  templateUrl: './route-tracker-item.component.html',
  styleUrls: ['./route-tracker-item.component.scss']
})
export class RouteTrackerItemComponent implements OnInit {

  @Input() section: RouteTrackerSection;

  constructor() { }

  ngOnInit(): void {
  }

}
