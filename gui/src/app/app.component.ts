import { Component, OnDestroy, OnInit } from '@angular/core';
import { NgcCookieConsentService, NgcInitializeEvent, NgcNoCookieLawEvent, NgcStatusChangeEvent } from 'ngx-cookieconsent';
import { Subscription } from 'rxjs';
import { CookieHandlerService } from './services/cookie-handler.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit, OnDestroy {
  title = 'gui';

  private statusChangeSubscription?: Subscription;
  private revokeChoiceSubscription?: Subscription;

  constructor(private ccService: NgcCookieConsentService, private cookieHandlerService: CookieHandlerService){}

  ngOnInit() {

    this.statusChangeSubscription = this.ccService.statusChange$.subscribe(
      (event: NgcStatusChangeEvent) => {
        this.cookieHandlerService.allowCookies(event.status === 'allow');
      });

    this.revokeChoiceSubscription = this.ccService.revokeChoice$.subscribe(
      () => {
        
        this.cookieHandlerService.allowCookies(false);
      });
  }

  ngOnDestroy() {
    this.statusChangeSubscription?.unsubscribe();
    this.revokeChoiceSubscription?.unsubscribe();
  }
}
