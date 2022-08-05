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
  private presetCategories = ["Sightseeing", "Other", "Nightlife", "Restaurants", "Shopping", "PicnicBarbequeSpot",
    "MuseumExhibition", "Nature", "Swimming"];
  public updateSuccessful = new EventEmitter<boolean>();

  constructor(private http: HttpClient) {
    this.backendUrl = environment.backendUrl;
    this.presetCategories.forEach((category) => {
      this.categories.push(new Category(category));
    });
  }

  public updateSights(coords: L.LatLng, radius: number) {
    const body = {
      "lat": coords["lat"],
      "lon": coords["lng"],
      "radius": radius
    }
    this.http.post(this.backendUrl + "/sights", body).subscribe((sights ) => {
      for (let category of this.categories) {
        category.sights = [];
      }
      this.sights = sights as Sight[];
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
      this.updateSuccessful.emit(true);
    }, (error => {
      this.updateSuccessful.emit(false);
    }));
  }

  public getSights() {
    return this.sights;
  }

  public getCategories(): Category[] {
    return this.categories;
  }
}
