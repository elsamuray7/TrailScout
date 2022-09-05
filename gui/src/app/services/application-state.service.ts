import { EventEmitter, Injectable } from '@angular/core';

@Injectable()
export class ApplicationStateService {

  private isMobileResolution: boolean;
  private routeModeActive = false;
  public routeModeChangedEvent = new EventEmitter<boolean>();

  constructor() {
    this. isMobileResolution = window.innerWidth < 768;
  }
  public getIsMobileResolution(): boolean {
    return this.isMobileResolution;
  }

  public isRouteModeActive(): boolean {
    return this.routeModeActive;
  }

  public toggleRouteMode(): void {
    this.routeModeActive = !this.routeModeActive;
    this.routeModeChangedEvent.emit(this.routeModeActive)
  }
}
