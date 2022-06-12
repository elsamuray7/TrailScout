import { Injectable } from '@angular/core';
import { CookieService } from 'ngx-cookie-service';
import { SightsPrios } from '../types.utils';
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
  getPriosCookies() {
    return this.cookieService.getAll();
  }
}
