import { Injectable } from '@angular/core';
import { CookieService } from 'ngx-cookie-service';
import { Cookie } from '../types.utils';
import { environment } from 'src/environments/environment';

@Injectable({
  providedIn: 'root'
})
export class CookieHandlerService {

  private cookiesAllowed = false;

  constructor(private cookieService: CookieService) { }

  allowCookies(value: boolean) {
    this.cookiesAllowed = value;
    if (!this.cookiesAllowed) {
      this.cookieService.deleteAll();
    }
  }

  setLocationCookie(latlng: L.LatLng) {
    if (!this.cookiesAllowed) {
      return;
    }
    const latlngString: string = JSON.stringify(latlng)
    this.cookieService.set(environment.cookieLocation, latlngString)
  }

  getLocationCookie(): Cookie {
    const result = this.cookieService.get(environment.cookieLocation);
    return {key: environment.cookieLocation, value: result} as Cookie;
  }

  setRadiusCookie(radius: number) {
    if (!this.cookiesAllowed) {
      return;
    }
    this.cookieService.set(environment.cookieRadius, radius.toString());
  }

  getRadiusCookie(): Cookie {
    const result = this.cookieService.get(environment.cookieRadius);
    return {key: environment.cookieRadius, value: result} as Cookie;
  }
}
