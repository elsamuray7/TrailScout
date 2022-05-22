import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';

@Component({
  selector: 'app-navigation-taskbar',
  templateUrl: './navigation-taskbar.component.html',
  styleUrls: ['./navigation-taskbar.component.scss']
})
export class NavigationTaskbarComponent implements OnInit {

  @Input() height: number = 50;
  @Output() startEvent = new EventEmitter;
  @Output() mainEvent = new EventEmitter;

  constructor() { }

  ngOnInit(): void {
  }

  startButtonClick() {
    this.startEvent.emit();
  }

  mainButtonClick() {
    this.mainEvent.emit();
  }

}
