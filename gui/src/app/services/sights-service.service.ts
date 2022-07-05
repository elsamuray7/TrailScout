import { EventEmitter, Injectable } from '@angular/core';
import { environment } from '../../environments/environment';
import { HttpClient } from '@angular/common/http';
import { Sight } from '../data/Sight';
import { Category } from '../data/Category';

@Injectable({
  providedIn: 'root'
})
export class SightsServiceService {
  private readonly backendUrl: String;
  private sights: Sight[];
  private categories: Category[] = [];
  public sightsChanged = new EventEmitter<Sight[]>();

  constructor(private http: HttpClient) {
    this.backendUrl = environment.backendUrl
  }

  public updateSights(coords: L.LatLng, radius: number) {
    const body = {
      "lat": coords["lat"],
      "lon": coords["lng"],
      "radius": radius
    }
    this.http.post(this.backendUrl + "/sights", body).subscribe((sights ) => {
      this.sights = sights as Sight[];
      this.sightsChanged.emit(this.sights);
      for (let sight of sights as Sight[]) {
        var categoryFound = false;
        for (let category of this.categories) {
          if (category.name === sight.category) {
            category.sights.push(sight);
            categoryFound = true;
          }
        }
        if (!categoryFound) {
          this.categories.push(new Category(sight.category))
          //add sight to this new category
          this.categories[this.categories.length-1].sights.push(sight);
        }
      }
    });
  }

  public getSights() {
    return this.sights;
  }
}
