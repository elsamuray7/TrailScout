import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { LandingPageComponent } from './landing-page.component';
import { NavigationModule } from '../navigation/navigation.module';
import { SwiperModule } from 'swiper/angular';


@NgModule({
  declarations: [
    LandingPageComponent
  ],
  imports: [
    CommonModule,
    NavigationModule,
    SwiperModule
  ]
})
export class LandingPageModule { }
