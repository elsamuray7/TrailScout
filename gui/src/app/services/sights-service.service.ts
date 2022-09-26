import { EventEmitter, Injectable } from '@angular/core';
import { environment } from '../../environments/environment';
import { HttpClient } from '@angular/common/http';
import { Sight } from '../data/Sight';
import { Category } from '../data/Category';
import {ToastService} from "./toast.service";

@Injectable({
  providedIn: 'root'
})
export class SightsServiceService {
  private readonly backendUrl: String;
  private categories: Category[] = [];
  private presetCategories = ["Sightseeing", "Activities", "Nightlife", "Restaurants", "Shopping", "PicnicBarbequeSpot",
    "MuseumExhibition", "Nature", "Swimming", "Animals"];
  public updating = new EventEmitter();
  public updateSuccessful = new EventEmitter<boolean>();

  public readonly categoryLabels = new Map<string, string>([
    ["Sightseeing", "Sehenswürdigkeiten"],
    ["Activities", "Aktivitäten"],
    ["Nightlife", "Nachtleben"],
    ["Restaurants", "Restaurants"],
    ["Shopping", "Shopping"],
    ["PicnicBarbequeSpot", "Picknick & Grillen"],
    ["MuseumExhibition", "Museen"],
    ["Nature", "Natur"],
    ["Swimming", "Badeplätze"],
    ["Animals", "Tiere"]
  ]);

  constructor(private http: HttpClient,
              private toastService: ToastService) {
    this.backendUrl = environment.backendUrl;
    this.presetCategories.forEach((category) => {
      this.categories.push(new Category(category));
    });
  }

  public updateSights(coords: L.LatLng, radius: number) {
    const body = {
      "lat": coords["lat"],
      "lon": coords["lng"],
      "radius": radius * 1000 // convert to meters
    }
    this.toastService.showStandard('Aktualisiere Sehenswürdigkeiten...');
    this.updating.emit();
    this.http.post(this.backendUrl + "/sights", body).subscribe((sights ) => {
      for (let category of this.categories) {
        // reduce sights array to only those with a Special Preference, those should be kept on refresh
        category.sights = category.getAllSightsWithSpecialPref();
      }
      for (let sight of sights as Sight[]) {
        for (let category of this.categories) {
          if (category.name === sight.category && !category.sights.includes(sight)
              && category.sights.findIndex(s => sight.node_id == s.node_id) == -1) {
            category.sights.push(sight);
          }
        }
      }
      this.toastService.showSuccess('Sehenswürdigkeiten erfolgreich aktualisiert!');
      this.updateSuccessful.emit(true);
    }, (error => {
      this.toastService.showDanger('Etwas ist schief gelaufen!');
      this.updateSuccessful.emit(false);
    }));
  }

  public getCategories(): Category[] {
    return this.categories;
  }
}
