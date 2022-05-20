import { Component, Input, OnInit } from '@angular/core';
import { Router } from '@angular/router';

@Component({
  selector: 'app-navigation-taskbar',
  templateUrl: './navigation-taskbar.component.html',
  styleUrls: ['./navigation-taskbar.component.scss']
})
export class NavigationTaskbarComponent implements OnInit {

  @Input() height: number = 200;

  constructor(private route: Router) { }

  ngOnInit(): void {
  }

  startPage() {
    this.route.navigate(['start']);
  }

  mainPage() {
    this.route.navigate(['main']);
  }

}
