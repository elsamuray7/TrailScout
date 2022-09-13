import { Component, OnInit, ViewEncapsulation  } from '@angular/core';
import SwiperCore, { Keyboard, Pagination, Navigation, Virtual } from 'swiper';
import { Router } from '@angular/router';

// install Swiper modules
SwiperCore.use([Keyboard, Pagination, Navigation, Virtual]);
@Component({
  selector: 'app-landing-page',
  templateUrl: './landing-page.component.html',
  styleUrls: ['./landing-page.component.scss'],
  encapsulation: ViewEncapsulation.None,
})
export class LandingPageComponent implements OnInit {

  constructor(private router: Router) { }

  ngOnInit(): void {
  }

  //Keine Ahnung ob man das Routing so macht
  goToPage(pageName:string){
    this.router.navigate([`${pageName}`]);
  }


}
