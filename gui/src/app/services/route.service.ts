import {EventEmitter, Injectable} from '@angular/core';
import {HttpClient} from "@angular/common/http";
import {environment} from "../../environments/environment";
import { catchError, of, timeout } from 'rxjs';
import { Sight } from '../data/Sight';


interface latlng {
  lat: number;
  lon: number;
}

interface route {
  type: string;
  time_budget: number;
  sight: Sight | null;
  nodes: latlng[];
}
export interface RouteResponse {
  route?: route[];
  error?: any
}


@Injectable({
  providedIn: 'root'
})
export class RouteService {

  private readonly backendUrl: String;
  private route?: RouteResponse;
  public routeUpdated = new EventEmitter<any>();
  public startRouteCall = new EventEmitter<any>();

  constructor(private http: HttpClient) {
    this.backendUrl = environment.backendUrl;
  }

  public async calculateRoute(request: any) {
    this.startRouteCall.emit();
    this.http.post(this.backendUrl + "/route", request).pipe(
      timeout(60000),
      catchError(e => {
        console.log(e);
        this.routeUpdated.emit({error: e, route: undefined} as RouteResponse);
        return of(null)
      })
      ).subscribe((route ) => {
      this.route = route as RouteResponse;
      if (this.route)  {
        this.routeUpdated.emit(route);
      }

    });
  }

  public getRoute() {
    return this.route;
  }
}
