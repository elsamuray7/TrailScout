import {EventEmitter, Injectable} from '@angular/core';
import {HttpClient} from "@angular/common/http";
import {environment} from "../../environments/environment";
import { catchError, of, Subject, timeout } from 'rxjs';
import { Sight } from '../data/Sight';
import {BaseBuilder, buildGPX} from 'gpx-builder';
import {Point} from "gpx-builder/dist/builder/BaseBuilder/models";

interface latlng {
  lat: number;
  lon: number;
}

interface route {
  type: string;
  time_budget: number;
  sight: Sight | null;
  nodes: latlng[];
  id?: number;
}
export interface RouteResponse {
  route?: route[];
  error?: any
}
export interface RouteRequest {
  start: string;
  end: string;
  walking_speed_kmh: number;
  area: {
    lat: number;
    lon: number;
    radius: number;
  }
  user_prefs: {
    categories: {
      category: string;
      pref: number;
    }[]
    sights: {
      id: number;
      name: string;
      category: string;
      pref: number;
    }[]
  }
}

@Injectable({
  providedIn: 'root'
})
export class RouteService {

  private readonly backendUrl: String;
  private route?: RouteResponse;
  private lastRequest?: RouteRequest;
  public routeUpdated = new EventEmitter<RouteResponse>();
  public startRouteCall = new EventEmitter<any>();
  public id$: Subject<number | null> = new Subject();
  public id_clicked$: Subject<number> = new Subject();

  constructor(private http: HttpClient) {
    this.backendUrl = environment.backendUrl;
    this.id$.next(null);
  }

  public async calculateRoute(request: RouteRequest) {
    this.startRouteCall.emit();
    this.http.post(this.backendUrl + "/route", request).pipe(
      timeout(300000),
      catchError(e => {
        console.log(e);
        this.routeUpdated.emit({error: e, route: undefined} as RouteResponse);
        return of(null)
      })
      ).subscribe((route ) => {
        this.lastRequest = request;
        this.route = route as RouteResponse;
        if (this.route)  {
          this.routeUpdated.emit(this.route);
        }

    });
  }

  public getRoute(): RouteResponse | null{
    return this.route ? this.route : null;
  }

  public getLastRequest(): RouteRequest | null {
    return this.lastRequest ? this.lastRequest : null;
  }


  public getRouteAsGpx():String | null{
    /**
     * Returns an XML string in GPX format from this.route
     * Returns null if things don't work out
     */

    if (this.route?.route === undefined) {
      return null;
    }
    const gpxData = new BaseBuilder();

    let trackPoints : Point[] = [];
    let wayPoints : Point[] = [];
    this.route.route.forEach(
      segment => {
        //if we have a sight add to waypoints
        if (segment.sight?.lon != undefined && segment.sight?.lat != undefined){
          wayPoints.push(new Point(<number>segment.sight?.lat, <number>segment.sight?.lon, {
            name :segment.sight.name
          }))
        }
        //all segment nodes for points in track
        segment.nodes.forEach(
          node =>{
            trackPoints.push(new Point(node.lat, node.lon));
          }
        )
      }
    )
    gpxData.setWayPoints(wayPoints);
    gpxData.setSegmentPoints(trackPoints);

    let gpxXmlString = buildGPX(gpxData.toObject());
    //console.log(gpxXmlString);
    return buildGPX(gpxData.toObject()) ? gpxXmlString : null;
  }
}
