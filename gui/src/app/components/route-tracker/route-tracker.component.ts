import { Component, Input, OnInit } from '@angular/core';
import { Category } from 'src/app/data/Category';
import { RouteTrackerSection } from 'src/app/types.utils';



@Component({
  selector: 'app-route-tracker',
  templateUrl: './route-tracker.component.html',
  styleUrls: ['./route-tracker.component.scss']
})
export class RouteTrackerComponent implements OnInit {

  @Input() sections!: RouteTrackerSection[];

  constructor() { }

  ngOnInit(): void {
  }

}
