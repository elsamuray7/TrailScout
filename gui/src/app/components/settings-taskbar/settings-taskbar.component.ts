import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { NgbTimeStruct } from '@ng-bootstrap/ng-bootstrap';
import { Settings, Tag, TagCheckboxResponse } from 'src/app/types.utils';

@Component({
  selector: 'app-settings-taskbar',
  templateUrl: './settings-taskbar.component.html',
  styleUrls: ['./settings-taskbar.component.scss']
})
export class SettingsTaskbarComponent implements OnInit {

  @Input() width: number = 200;
  @Input() tags: Tag[] = [];

  @Output() settings = new EventEmitter;
  @Output() radiusChange = new EventEmitter;

  private _radius!: number;
  private _startTime: NgbTimeStruct;
  private _walkTime?: NgbTimeStruct;
  private _endTime?: NgbTimeStruct;
  private currentDate: Date;
  private selectedTags: Tag[] = [];

  constructor() {
    this.currentDate = new Date();
    this._startTime = {hour: this.currentDate.getHours(), minute: this.currentDate.getMinutes(), second: 0};
   }

  ngOnInit(): void {
  }

  set radius(r: number) {
    this._radius = r;
    this.radiusChange.emit(r);
  }

  get radius() {
    return this._radius;
  }

  set startTime(time: NgbTimeStruct) {
    this._startTime = time;
    const totalMinutes = this.ngbTimeStructToMinutes(this.startTime) + this.ngbTimeStructToMinutes(this.walkTime);
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    this.endTime = {hour: hours, minute: minutes, second: 0 };
  }

  get startTime() {
    return this._startTime!;
  }

  set walkTime(time: NgbTimeStruct) {
    this._walkTime = time;
    const totalMinutes = this.ngbTimeStructToMinutes(this.startTime) + this.ngbTimeStructToMinutes(this.walkTime);
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    this.endTime = {hour: hours, minute: minutes, second: 0 };
  }

  get walkTime() {
    return this._walkTime!;
  }

  set endTime(time: NgbTimeStruct) {
    this._endTime = time;
  }

  get endTime() {
    return this._endTime!;
  }

  calculationAllowed() {
    if (this.radius || this.walkTime) {
      return true;
    }
    return false;
  }

  calculate(){
    const result: Settings = {
      radius: this.radius,
      startTime: this.startTime,
      walkTime: this.walkTime,
      endTime: this.endTime,
      tags: this.selectedTags
    }
    this.settings.emit(result)
  }

  ngbTimeStructToMinutes(time: NgbTimeStruct) {
    if (!time) {
      return 0;
    }
    return time.minute + time.hour * 60;
  }

  checkedTag(response: TagCheckboxResponse) {
    if (response.checked) {
      if (!this.selectedTags.includes(response.tag)) {
        this.selectedTags.push(response.tag);
      }
    } else {
      const index = this.selectedTags.indexOf(response.tag);
      if (index > -1) {
        this.selectedTags.splice(index, 1)
      }
    }
  }

}
