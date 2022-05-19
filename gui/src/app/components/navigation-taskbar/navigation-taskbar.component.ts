import { Component, Input, OnInit } from '@angular/core';

@Component({
  selector: 'app-navigation-taskbar',
  templateUrl: './navigation-taskbar.component.html',
  styleUrls: ['./navigation-taskbar.component.scss']
})
export class NavigationTaskbarComponent implements OnInit {

  @Input() height: number = 200;

  constructor() { }

  ngOnInit(): void {
  }

}
