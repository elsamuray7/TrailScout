import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { RouteTrackerSection } from 'src/app/types.utils';

@Component({
  selector: 'app-route-tracker-item',
  templateUrl: './route-tracker-item.component.html',
  styleUrls: ['./route-tracker-item.component.scss']
})
export class RouteTrackerItemComponent implements OnInit {

  @Input() section?: RouteTrackerSection;
  @Input() currentSection: boolean = false;
  @Output() hoverEvent = new EventEmitter;

  constructor() { }

  ngOnInit(): void {
    if (this.currentSection) {
      this.hoverEvent.emit(this.section?.routeId);
    }
  }

  hover() {
    this.hoverEvent.emit(this.section!.routeId);
    console.log(this.section);
  }

  unhover() {
    this.hoverEvent.emit(null);
  }
}
