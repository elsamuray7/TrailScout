import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { PagesModule } from './pages/pages.module';
import { NgbModule } from '@ng-bootstrap/ng-bootstrap';
import { CookieService } from 'ngx-cookie-service';
import {NgcCookieConsentModule, NgcCookieConsentConfig} from 'ngx-cookieconsent';
import { CookieHandlerService } from './services/cookie-handler.service';
import { HttpClientModule } from '@angular/common/http';
import { BlockUIModule } from 'ng-block-ui';
import { environment } from 'src/environments/environment';
import {ComponentsModule} from "./components/components.module";
import { SwiperModule } from 'swiper/angular';

const cookieConfig:NgcCookieConsentConfig = {
  cookie: {
    domain: environment.domain
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
        PagesModule,
        HttpClientModule,
        NgbModule,
        NgcCookieConsentModule.forRoot(cookieConfig),
        NgcCookieConsentModule,
        BlockUIModule.forRoot({
            delayStart: 200,
            delayStop: 500
        }),
        ComponentsModule,
        SwiperModule
    ],
  providers: [CookieService, CookieHandlerService],
  bootstrap: [AppComponent]
})
export class AppModule { }
