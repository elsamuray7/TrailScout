import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { LandingPageComponent } from './landing-page.component';
import { NavigationModule } from '../navigation/navigation.module';
import { ButtonsModule } from 'ngx-foundation';



@NgModule({
  declarations: [
    LandingPageComponent
  ],
  imports: [
    CommonModule,
    NavigationModule
  ]
})
export class LandingPageModule { }
