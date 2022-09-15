import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';

@Component({
  selector: 'app-navigation',
  templateUrl: './navigation.component.html',
  styleUrls: ['./navigation.component.scss']
})
export class NavigationComponent implements OnInit {

  constructor(private route: Router) { }

  ngOnInit(): void {
  }

  startPage() {
    this.route.navigate(['start']);
  }

  mainPage() {
    this.route.navigate(['scout']);
  }
}
