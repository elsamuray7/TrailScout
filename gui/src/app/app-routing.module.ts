import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { LandingPageComponent } from './desktop/landing-page/landing-page.component';
import { MainPageComponent } from './desktop/main-page/main-page.component';
import { PageNotFoundComponent } from './desktop/page-not-found/page-not-found.component';

const routes: Routes = [
  {path: '', redirectTo: 'start', pathMatch: 'full'},
  {path: 'start', component: LandingPageComponent},
  {path: 'main', component: MainPageComponent},
  {path: '**', pathMatch: 'full', component: PageNotFoundComponent}
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule { }
