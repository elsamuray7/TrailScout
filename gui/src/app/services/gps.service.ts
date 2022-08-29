import { Injectable } from '@angular/core';
import { LatLng } from 'leaflet';
import { Observable } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class GPSService {


    getLocation() {
        return new Observable(observer => {
            navigator.geolocation.watchPosition(pos => {
                observer.next(new LatLng(pos.coords.latitude, pos.coords.longitude));
            })
        })
    }

    async getCurrentLocation(): Promise<L.LatLng> {
        return new Promise((resolve: any, reject: any) => {
            navigator.geolocation.getCurrentPosition(pos => {
                if (pos) {
                    resolve(new LatLng(pos.coords.latitude, pos.coords.longitude))
                } else {
                    alert("Geolocation is not supported by this browser.");
                }
                
            },
            (error: any) => reject(error));
        })
        
    }

}