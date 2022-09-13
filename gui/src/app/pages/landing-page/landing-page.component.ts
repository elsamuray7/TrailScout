import { Component, OnInit, ViewEncapsulation  } from '@angular/core';
import SwiperCore, { Keyboard, Pagination, Navigation, Virtual } from 'swiper';
import { BehaviorSubject } from 'rxjs';

// install Swiper modules
SwiperCore.use([Keyboard, Pagination, Navigation, Virtual]);
@Component({
  selector: 'app-landing-page',
  templateUrl: './landing-page.component.html',
  styleUrls: ['./landing-page.component.scss'],
  encapsulation: ViewEncapsulation.None,
})
export class LandingPageComponent implements OnInit {

  constructor() { }

  ngOnInit(): void {
  }


}
