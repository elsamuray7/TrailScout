import {EventEmitter, Injectable} from '@angular/core';
import {HttpClient} from "@angular/common/http";
import {environment} from "../../environments/environment";


interface latlng {
  lat: number;
  lon: number;
}

interface sight {
  sight?: latlng;
  nodes: latlng[];
}
export interface RouteResponse {
  route: sight[];
}

@Injectable({
  providedIn: 'root'
})
export class RouteService {

  private readonly backendUrl: String;
  private route?: RouteResponse;
  public routeUpdated = new EventEmitter<any>();

  constructor(private http: HttpClient) {
    this.backendUrl = environment.backendUrl;
  }

  public async calculateRoute(request: any) {
    this.http.post(this.backendUrl + "/route", request).subscribe((route ) => {
      this.route = route as RouteResponse;
      if (this.route)  {
        console.log(route);
        this.routeUpdated.emit(route);
      }
      
    });
  }

  public getRoute() {
    return this.route;
  }
}
