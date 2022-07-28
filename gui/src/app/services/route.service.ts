import {EventEmitter, Injectable} from '@angular/core';
import {HttpClient} from "@angular/common/http";
import {environment} from "../../environments/environment";

@Injectable({
  providedIn: 'root'
})
export class RouteService {

  private readonly backendUrl: String;
  private route: any;
  public routeUpdated = new EventEmitter<any>();

  constructor(private http: HttpClient) {
    this.backendUrl = environment.backendUrl;
  }

  public calculateRoute(request: any) {
    this.http.post(this.backendUrl + "/route", request).subscribe((route ) => {
      this.route = route;
      this.routeUpdated.emit(route);
    });
  }

  public getRoute() {
    return this.route;
  }
}
