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
  @Output() clickEvent = new EventEmitter;

  _hover = false;

  constructor() { }

  ngOnInit(): void {
    if (this.currentSection) {
      this.hoverEvent.emit(this.section?.routeId);
    }
  }

  hover() {
    this.hoverEvent.emit(this.section?.routeId);
    this._hover = true;
  }

  unhover() {
    this.hoverEvent.emit(null);
    this._hover = false;
  }

  onClick() {
    this.clickEvent.emit(this.section?.routeId)
  }
}
