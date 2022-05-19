import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationTaskbarComponent } from './navigation-taskbar.component';
import { ButtonsModule } from 'ngx-foundation';



@NgModule({
  declarations: [
    NavigationTaskbarComponent
  ],
  imports: [
    CommonModule,
    ButtonsModule
  ],
  exports: [
    NavigationTaskbarComponent
  ]
})
export class NavigationTaskbarModule { }
