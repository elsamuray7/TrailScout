import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationTaskbarComponent } from './navigation-taskbar/navigation-taskbar.component';
import { SettingsTaskbarComponent } from './settings-taskbar/settings-taskbar.component';
import { MapContainerComponent } from './map-container/map-container.component';
import { FormsModule } from '@angular/forms';
import { NgbModule } from '@ng-bootstrap/ng-bootstrap';
import { SettingsTaskbarTagItemComponent } from './settings-taskbar/settings-taskbar-tag-item/settings-taskbar-tag-item.component';
import { ToastComponent } from './toast/toast.component';
import { RouteTrackerComponent } from './route-tracker/route-tracker.component';


@NgModule({
  declarations: [
    NavigationTaskbarComponent,
    SettingsTaskbarComponent,
    MapContainerComponent,
    SettingsTaskbarTagItemComponent,
    ToastComponent,
    RouteTrackerComponent
  ],
  imports: [
    CommonModule,
    FormsModule,
    NgbModule
  ],
  exports: [
    NavigationTaskbarComponent,
    SettingsTaskbarComponent,
    MapContainerComponent,
    ToastComponent
  ],
  bootstrap: [
    SettingsTaskbarComponent
  ]
})
export class ComponentsModule { }
