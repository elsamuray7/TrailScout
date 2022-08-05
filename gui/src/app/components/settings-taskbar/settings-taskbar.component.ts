import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { NgbTimeStruct } from '@ng-bootstrap/ng-bootstrap';
import { Settings } from 'src/app/types.utils';
import { SightsServiceService } from '../../services/sights-service.service';
import { CookieHandlerService } from '../../services/cookie-handler.service';
import {Category} from "../../data/Category";
import { ToastService } from '../../services/toast.service';

@Component({
  selector: 'app-settings-taskbar',
  templateUrl: './settings-taskbar.component.html',
  styleUrls: ['./settings-taskbar.component.scss']
})
export class SettingsTaskbarComponent implements OnInit {

  @Input() width: number = 200;
  @Input() startPointSet = false;
  @Input() startRadius? :number;

  @Output() radiusChange = new EventEmitter;
  @Output() closeButton = new EventEmitter;
  @Output() drawSightsEvent = new EventEmitter;

  public _radius!: number;
  private _startTime: NgbTimeStruct;
  private _walkTime?: NgbTimeStruct;
  private _endTime?: NgbTimeStruct;
  private currentDate: Date;
  refreshing: boolean = false;

  constructor(private sightsService: SightsServiceService,
              private cookieService: CookieHandlerService,
              private toastService: ToastService) {
    this.currentDate = new Date();
    this._startTime = {hour: this.currentDate.getHours(), minute: this.currentDate.getMinutes(), second: 0};
   }

  ngOnInit(): void {
    //Temp fix as angular throws expressionChangedAfterChecked error
    setTimeout(()=> {
      if (this.startRadius) {
        this.radius = this.startRadius;
      }
  }, 0);

    this.sightsService.updateSuccessful.subscribe((success) => {
      this.refreshing = false;
      if (success) {
        this.toastService.showSuccess('Successfully updated sights!');
      } else {
        this.toastService.showDanger('Something went wrong!');
      }
    });
  }

  set radius(r: number) {
    this._radius = r;
    this.radiusChange.emit(r);
  }

  get radius() {
    return this._radius;
  }

  public getCategories(): Category[] {
    return this.sightsService.getCategories();
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
    return (this.radius > 0 || this.walkTime) && this.startPointSet;
  }

  calculate(){
    // TODO: Routen Request wird hier gestartet
    const result: Settings = {
      radius: this.radius,
      startTime: this.startTime,
      walkTime: this.walkTime,
      endTime: this.endTime
    }
  }

  ngbTimeStructToMinutes(time: NgbTimeStruct) {
    if (!time) {
      return 0;
    }
    return time.minute + time.hour * 60;
  }

  drawSights(drawSight: boolean, category: Category) {
    const response = {
      "drawSight": drawSight,
      "category": category
    }
    this.drawSightsEvent.emit(response);
  }

  close() {
    this.closeButton.emit();
  }

  refreshSights() {
    const startCookie = this.cookieService.getLocationCookie();
    if (startCookie.value !== '' && this.radius > 0) {
      const val = startCookie.value as string;
      const coords = JSON.parse(val);
      this.refreshing = true;
      this.toastService.showStandard('Updating sights...');
      this.sightsService.updateSights(coords, this.radius);
    }
  }
}
