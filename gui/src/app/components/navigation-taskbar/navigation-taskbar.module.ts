import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationTaskbarComponent } from './navigation-taskbar.component';



@NgModule({
  declarations: [
    NavigationTaskbarComponent
  ],
  imports: [
    CommonModule
  ],
  exports: [
    NavigationTaskbarComponent
  ]
})
export class NavigationTaskbarModule { }
