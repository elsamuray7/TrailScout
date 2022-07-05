import { Injectable } from '@angular/core';
import { CookieService } from 'ngx-cookie-service';
import { Cookie, SightsPrios } from '../types.utils';
import { environment } from 'src/environments/environment';

@Injectable({
  providedIn: 'root'
})
export class CookieHandlerService {

  private cookiesAllowed = false;

  constructor(private cookieService: CookieService) { }

  allowCookies(value: boolean) {
    this.cookiesAllowed = value;
    if (this.cookiesAllowed === false) {
      this.cookieService.deleteAll();
    }
  }

  setPriosCookie(prios: SightsPrios) {
    if (!this.cookiesAllowed) {
      return;
    }
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
    if (!this.cookiesAllowed) {
      return;
    }
    const latlngString: string = latlng.toString();
    this.cookieService.set(environment.cookieLocation, latlngString)
  }

  getLocationCookie(): Cookie {
    const result = this.cookieService.get(environment.cookieLocation);
    const cookie: Cookie = {key: environment.cookieLocation, value: result}
    return cookie;
  }

  setRadiusCookie(radius: number) {
    if (!this.cookiesAllowed) {
      return;
    }
    this.cookieService.set(environment.cookieRadius, radius.toString());
  }

  getRadiusCookie(): Cookie {
    const result = this.cookieService.get(environment.cookieRadius);
    const cookie: Cookie = {key: environment.cookieRadius, value: result};
    return cookie;
  }
}
