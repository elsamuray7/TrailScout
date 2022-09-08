import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MainPageComponent } from './main-page.component';
import { NavigationModule } from '../navigation/navigation.module';
import { ComponentsModule } from 'src/app/components/components.module';
import { NgbCollapseModule, NgbModule, NgbTooltipModule } from '@ng-bootstrap/ng-bootstrap';
import { ApplicationStateService } from 'src/app/services/application-state.service';
import { BlockUIModule } from 'ng-block-ui';
import { GPSService } from 'src/app/services/gps.service';



@NgModule({
  declarations: [
    MainPageComponent
  ],
  imports: [
    CommonModule,
    NavigationModule,
    ComponentsModule,
    NgbModule,
    BlockUIModule
  ],
  providers: [ApplicationStateService, GPSService]
})
export class MainPageModule { }
