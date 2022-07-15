import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { DesktopModule } from './pages/desktop.module';
import { NavigationModule } from './pages/navigation/navigation.module';
import { NgbModule } from '@ng-bootstrap/ng-bootstrap';
import { CookieService } from 'ngx-cookie-service';
import {NgcCookieConsentModule, NgcCookieConsentConfig} from 'ngx-cookieconsent';
import { CookieHandlerService } from './services/cookie-handler.service';
import { HttpClientModule } from '@angular/common/http';

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
    HttpClientModule,
    NavigationModule,
    NgbModule,
    NgcCookieConsentModule.forRoot(cookieConfig),
    NgcCookieConsentModule
  ],
  providers: [CookieService, CookieHandlerService],
  bootstrap: [AppComponent]
})
export class AppModule { }
