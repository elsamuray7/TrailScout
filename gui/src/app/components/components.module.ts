import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationTaskbarComponent } from './navigation-taskbar/navigation-taskbar.component';
import { SettingsTaskbarComponent } from './settings-taskbar/settings-taskbar.component';
import { MapContainerComponent } from './map-container/map-container.component';
import { FormsModule } from '@angular/forms';
import { NgbModule } from '@ng-bootstrap/ng-bootstrap';



@NgModule({
  declarations: [
    NavigationTaskbarComponent,
    SettingsTaskbarComponent,
    MapContainerComponent
  ],
  imports: [
    CommonModule,
    FormsModule,
    NgbModule
  ],
  exports: [
    NavigationTaskbarComponent,
    SettingsTaskbarComponent,
    MapContainerComponent
  ],
  bootstrap: [
    SettingsTaskbarComponent
  ]
})
export class ComponentsModule { }
