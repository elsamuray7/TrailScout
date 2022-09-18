import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MainPageComponent } from './main-page.component';
import { ComponentsModule } from 'src/app/components/components.module';
import { NgbModule } from '@ng-bootstrap/ng-bootstrap';
import { ApplicationStateService } from 'src/app/services/application-state.service';
import { BlockUIModule } from 'ng-block-ui';
import { GPSService } from 'src/app/services/gps.service';



@NgModule({
  declarations: [
    MainPageComponent
  ],
  imports: [
    CommonModule,
    ComponentsModule,
    NgbModule,
    BlockUIModule
  ],
  providers: [ApplicationStateService, GPSService]
})
export class MainPageModule { }
