import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationComponent } from './navigation.component';
import { ComponentsModule } from 'src/app/components/components.module';



@NgModule({
  declarations: [
    NavigationComponent
  ],
  imports: [
    CommonModule,
    ComponentsModule
  ],
  exports: [
    NavigationComponent
  ]
})
export class NavigationModule { }
