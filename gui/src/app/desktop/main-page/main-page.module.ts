import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MainPageComponent } from './main-page.component';
import { NavigationModule } from '../navigation/navigation.module';
import { ComponentsModule } from 'src/app/components/components.module';
import { NgbCollapseModule } from '@ng-bootstrap/ng-bootstrap';



@NgModule({
  declarations: [
    MainPageComponent
  ],
  imports: [
    CommonModule,
    NavigationModule,
    ComponentsModule,
    NgbCollapseModule
  ]
})
export class MainPageModule { }
