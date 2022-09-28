import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MainPageModule } from './main-page/main-page.module';
import { PageNotFoundComponent } from './page-not-found/page-not-found.component';
import { LandingPageModule } from './landing-page/landing-page.module';



@NgModule({
  declarations: [
    PageNotFoundComponent
  ],
  imports: [
    CommonModule,
    MainPageModule,
    LandingPageModule
  ]
})
export class PagesModule { }
