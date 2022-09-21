import { Component, OnInit, ViewEncapsulation  } from '@angular/core';
import SwiperCore, { Keyboard, Pagination, Navigation, Virtual } from 'swiper';
import { Router } from '@angular/router';
import { ApplicationStateService } from 'src/app/services/application-state.service';

// install Swiper modules
SwiperCore.use([Keyboard, Pagination, Navigation, Virtual]);
@Component({
  selector: 'app-landing-page',
  templateUrl: './landing-page.component.html',
  styleUrls: ['./landing-page.component.scss'],
  encapsulation: ViewEncapsulation.None,
})
export class LandingPageComponent implements OnInit {

  mobile = false;

  constructor(private router: Router, private mobileState: ApplicationStateService) { }

  ngOnInit(): void {
    this.mobile = this.mobileState.getIsMobileResolution();
  }

  //Keine Ahnung ob man das Routing so macht
  goToPage(pageName:string){
    this.router.navigate([pageName]);
  }


}
