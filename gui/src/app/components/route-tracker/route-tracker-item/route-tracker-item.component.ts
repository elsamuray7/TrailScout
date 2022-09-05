import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { RouteTrackerSection } from 'src/app/types.utils';
import { getIcon } from '../../icons';

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
    console.log(this.section)
  }

  lastSection() {
    return this.section?.sight === null;
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

  image() {
    if (!this.section || !this.section.sight) {
      return 'assets/icons/start.png';
    }
    return getIcon(this.section.sight).options.iconUrl;
  }
}
