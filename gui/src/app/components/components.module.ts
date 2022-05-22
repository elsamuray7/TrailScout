import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationTaskbarComponent } from './navigation-taskbar/navigation-taskbar.component';
import { SettingsTaskbarComponent } from './settings-taskbar/settings-taskbar.component';
import { MapContainerComponent } from './map-container/map-container.component';



@NgModule({
  declarations: [
    NavigationTaskbarComponent,
    SettingsTaskbarComponent,
    MapContainerComponent
  ],
  imports: [
    CommonModule
  ],
  exports: [
    NavigationTaskbarComponent,
    SettingsTaskbarComponent,
    MapContainerComponent
  ]
})
export class ComponentsModule { }
