import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { DesktopModule } from './desktop/desktop.module';
import { NavigationModule } from './desktop/navigation/navigation.module';
import { MobileModule } from './mobile/mobile.module';
import { NgbModule } from '@ng-bootstrap/ng-bootstrap';
import { CookieService } from 'ngx-cookie-service';
import {NgcCookieConsentModule, NgcCookieConsentConfig} from 'ngx-cookieconsent';
import { CookieHandlerService } from './services/cookie-handler.service';

const cookieConfig:NgcCookieConsentConfig = {
  cookie: {
    domain: 'localhost' // or 'your.domain.com' // it is mandatory to set a domain, for cookies to work properly (see https://goo.gl/S2Hy2A)
  },
  palette: {
    popup: {
      background: '#000'
    },
    button: {
      background: '#f1d600'
    }
  },
  theme: 'edgeless',
  type: 'opt-out'
};

@NgModule({
  declarations: [
    AppComponent
  ],
  imports: [
    BrowserModule,
    AppRoutingModule,
    DesktopModule,
    MobileModule,
    NavigationModule,
    NgbModule,
    NgcCookieConsentModule.forRoot(cookieConfig),
    NgcCookieConsentModule
  ],
  providers: [CookieService, CookieHandlerService],
  bootstrap: [AppComponent]
})
export class AppModule { }
