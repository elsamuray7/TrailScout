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
  private _walkTime: NgbTimeStruct;
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
    this._walkTime = {hour: 1, minute: 0, second: 0};
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
      "end": this.transformTimeToISO8601Date(this._endTime),
      "walking_speed_kmh": 50,
      "area": {
        "lat": this.mapService.getCoordniates().lat,
        "lon": this.mapService.getCoordniates().lng,
        "radius": this.mapService.getRadius()
      },
      "user_prefs": {
        "categories": categories,
        "sights": []
      }
    }
    this.routeService.calculateRoute(request);
  }

  transformTimeToISO8601Date(time: NgbTimeStruct): string {
    var result = this.currentDate.getFullYear().toString() + "-";
    if (this.currentDate.getMonth() < 10) {
      result += "0" + this.currentDate.getMonth().toString() + "-";
    } else {
      result += this.currentDate.getMonth().toString() + "-";
    }
    if (this.currentDate.getDate() < 10) {
      result += "0" + this.currentDate.getDate().toString() + "T";
    } else {
      result += this.currentDate.getDate().toString() + "T";
    }
    if (time.hour < 10) {
      result += "0" + time.hour + ":";
    } else {
      result += time.hour + ":";
    }
    if (time.minute < 10) {
      result += "0" + time.minute + ":";
    } else {
      result += time.minute + ":";
    }
    if (time.second < 10) {
      result += "0" + time.second + "Z";
    } else {
      result += time.second + "Z";
    }
    return result;
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
