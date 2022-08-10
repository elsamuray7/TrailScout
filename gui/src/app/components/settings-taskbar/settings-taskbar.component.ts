import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { NgbTimeStruct } from '@ng-bootstrap/ng-bootstrap';
import { SightsServiceService } from '../../services/sights-service.service';
import {Category} from "../../data/Category";
import {MapService} from "../../services/map.service";
import {RouteService} from "../../services/route.service";
import { ToastService } from '../../services/toast.service';
import { CookieHandlerService } from 'src/app/services/cookie-handler.service';

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
  private _endTime: NgbTimeStruct;
  private currentDate: Date;
  refreshing: boolean = false;

  constructor(private sightsService: SightsServiceService,
              private mapService: MapService,
              private routeService: RouteService,
              private cookieService: CookieHandlerService,
              private toastService: ToastService) {
    this.currentDate = new Date();
    this._startTime = {hour: this.currentDate.getHours(), minute: this.currentDate.getMinutes(), second: 0};
    this._endTime = {hour: this.startTime.hour + 1, minute: this.startTime.minute, second: this.startTime.second};
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
  }

  get startTime() {
    return this._startTime!;
  }

  set endTime(time: NgbTimeStruct) {
    this._endTime = time;
  }

  get endTime() {
    return this._endTime!;
  }

  calculationAllowed() {
    return this.radius > 0 && this.startPointSet;
  }

  async calculate(){
    var categories: any[] = [];
    this.sightsService.getCategories().forEach((category) => {
      if (category.pref > 0) {
        categories.push({
          "name": category.name,
          "pref": category.pref
        })
      }
    })
    const request = {
      "start": this.transformTimeToISO8601Date(this._startTime),
      "end": this.transformTimeToISO8601Date(this._endTime, !this.isStartBeforeEnd()),
      "walking_speed_kmh": 50,
      "area": {
        "lat": this.mapService.getCoordniates().lat,
        "lon": this.mapService.getCoordniates().lng,
        "radius": this.mapService.getRadius() * 1000
      },
      "user_prefs": {
        "categories": categories,
        "sights": []
      }
    }
    this.routeService.calculateRoute(request);
  }

  transformTimeToISO8601Date(time: NgbTimeStruct, nextDay = false): string {
    var tempDate = this.currentDate;
    tempDate.setUTCHours(time.hour, time.minute, time.second);
    if(nextDay) {
      tempDate.setDate(tempDate.getDate() +1);
    }
    return tempDate.toISOString();
  }

  isStartBeforeEnd(): boolean {
    return this._startTime.hour < this._endTime.hour ||
      (this._startTime.hour == this._endTime.hour && this._startTime.minute < this._endTime.minute);
  }

  getMinutesBetweenStartAndEnd() {
    if (this.isStartBeforeEnd()) {
      // if start before end
      return (this._endTime.hour - this._startTime.hour) * 60 + this._endTime.minute - this._startTime.minute
    } else {
      // if start after end => end is on the next day
      return (23 - this._startTime.hour) * 60 + (60 - this._startTime.minute)
        + this._endTime.hour * 60 + this._endTime.minute;
    }
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
