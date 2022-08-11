import { Component, OnInit } from '@angular/core';

@Component({
  selector: 'app-screenshot-carousel',
  templateUrl: './screenshot-carousel.component.html',
  styleUrls: ['./screenshot-carousel.component.scss']
})
export class ScreenshotCarouselComponent implements OnInit {

  images = [
    'assets/screenshot_1.png',
    'assets/screenshot_2.png'
  ]

  constructor() { }

  ngOnInit(): void {
  }

}
