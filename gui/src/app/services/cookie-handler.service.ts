import { Injectable } from '@angular/core';
import { CookieService } from 'ngx-cookie-service';
import { Cookie, SightsPrios } from '../types.utils';
import { environment } from 'src/environments/environment';

@Injectable({
  providedIn: 'root'
})
export class CookieHandlerService {

  constructor(private cookieService: CookieService) { }

  setPriosCookie(prios: SightsPrios) {
    for (const prio of prios.sightWithPrio) {
      const cookie = prio[0];
      this.cookieService.set(cookie, prio[1].toString());
    }
  }
  getPriosCookies(): Cookie[] {
    const result =  this.cookieService.getAll();
    const prioCookies: Cookie[] = [];
    Object.entries(result).forEach(res => {
      prioCookies.push({key: res[0], value: res[1] as any})
    });
    return prioCookies;
  }

  setLocationCookie(latlng: L.LatLng) {
    const latlngString: string = JSON.stringify(latlng)
    this.cookieService.set(environment.cookieLocation, latlngString)
  }

  getLocationCookie(): Cookie {
    const result = this.cookieService.get(environment.cookieLocation);
    return {key: environment.cookieLocation, value: result} as Cookie;
  }

  setRadiusCookie(radius: number) {
    this.cookieService.set(environment.cookieRadius, radius.toString());
  }

  getRadiusCookie(): Cookie {
    const result = this.cookieService.get(environment.cookieRadius);
    return {key: environment.cookieRadius, value: result} as Cookie;
  }
}
