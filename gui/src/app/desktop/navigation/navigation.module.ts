import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationComponent } from './navigation.component';
import { NavigationTaskbarModule } from 'src/app/components/navigation-taskbar/navigation-taskbar.module';



@NgModule({
  declarations: [
    NavigationComponent
  ],
  imports: [
    CommonModule,
    NavigationTaskbarModule
  ],
  exports: [
    NavigationComponent
  ]
})
export class NavigationModule { }
