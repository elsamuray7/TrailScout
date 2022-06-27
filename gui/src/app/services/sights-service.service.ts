import { Injectable } from '@angular/core';
import { environment } from '../../environments/environment';
import { HttpClient } from '@angular/common/http';

@Injectable({
  providedIn: 'root'
})
export class SightsServiceService {
  private readonly backendUrl: String;

  constructor(private http: HttpClient) {
    this.backendUrl = environment.backendUrl
  }

  public getSights(coords: L.LatLng, radius: number) {
    const body = {
      "lat": coords["lat"],
      "lon": coords["lng"],
      "radius": radius
    }
    return this.http.post(this.backendUrl + "/sights", body);
  }
}
