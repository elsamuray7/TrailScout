import { Component, Input, OnInit } from '@angular/core';

@Component({
  selector: 'app-settings-taskbar',
  templateUrl: './settings-taskbar.component.html',
  styleUrls: ['./settings-taskbar.component.scss']
})
export class SettingsTaskbarComponent implements OnInit {

  @Input() width: number = 200;

  calculationAllowed = false;
  radius: number = 30;

  constructor() { }

  ngOnInit(): void {
  }

}
