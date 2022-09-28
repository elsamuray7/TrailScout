import { Component, Input, OnInit } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { Category } from 'src/app/data/Category';
import { RouteService } from 'src/app/services/route.service';
import { RouteTrackerSection } from 'src/app/types.utils';



@Component({
  selector: 'app-route-tracker',
  templateUrl: './route-tracker.component.html',
  styleUrls: ['./route-tracker.component.scss']
})
export class RouteTrackerComponent implements OnInit {

  @Input() sections!: RouteTrackerSection[];
  @Input() userLocation?: L.LatLng;

 

  constructor(private routeService: RouteService) {
   }

  ngOnInit(): void {
  }

  getClosestSection() {
    if (!this.userLocation) {
      return undefined;
    }
    let minimum = Infinity;
    let minId = undefined;
    this.sections.forEach(section => {  
      section.section.forEach(nodes => {
        const min = this.getDistanceFromLatLonInKm(this.userLocation!.lat, this.userLocation!.lng, nodes.lat, nodes.lng);
        if (minimum > min) {
          minimum = min;
          minId = section.routeId;
        }
      })
    })
    return minId;
  }

  getDistanceFromLatLonInKm(lat1: number, lon1: number, lat2: number, lon2: number) {

    let p = 0.017453292519943295;    // Math.PI / 180
    let c = Math.cos;
    let a = 0.5 - c((lat2 - lat1) * p) / 2 +
      c(lat1 * p) * c(lat2 * p) *
      (1 - c((lon2 - lon1) * p)) / 2;

    return 12742 * Math.asin(Math.sqrt(a)) * 1000; // 2 * R; R = 6371 km
  }

  hoveredItem(id: number | null) {
      this.routeService.id$.next(id);
  }

  clickedItem(id: number) {
    this.routeService.id_clicked$.next(id);
  }
}
