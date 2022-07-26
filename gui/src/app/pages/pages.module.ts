import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationModule } from './navigation/navigation.module';
import { MainPageModule } from './main-page/main-page.module';
import { LandingPageModule } from './landing-page/landing-page.module';
import { PageNotFoundComponent } from './page-not-found/page-not-found.component';



@NgModule({
  declarations: [
    PageNotFoundComponent
  ],
  imports: [
    CommonModule,
    NavigationModule,
    MainPageModule
  ]
})
export class PagesModule { }
