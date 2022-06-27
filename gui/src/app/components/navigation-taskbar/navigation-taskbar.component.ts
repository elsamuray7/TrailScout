import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';

@Component({
  selector: 'app-navigation-taskbar',
  templateUrl: './navigation-taskbar.component.html',
  styleUrls: ['./navigation-taskbar.component.scss']
})
export class NavigationTaskbarComponent implements OnInit {

  @Output() startEvent = new EventEmitter;
  @Output() mainEvent = new EventEmitter;

  themes: String[] = [
    "Cyborg",
    "Cosmo",
    "Flatly"
  ]
  constructor() { }

  ngOnInit(): void {
  }

  startButtonClick() {
    this.startEvent.emit();
  }

  mainButtonClick() {
    this.mainEvent.emit();
  }

  changeTheme(theme: String) {
    console.log('changing theme to:' + theme);

  }
}
