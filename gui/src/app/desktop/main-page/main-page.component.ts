import { Component, OnInit } from '@angular/core';
import { Tag } from 'src/app/types.utils';

@Component({
  selector: 'app-main-page',
  templateUrl: './main-page.component.html',
  styleUrls: ['./main-page.component.scss']
})
export class MainPageComponent implements OnInit {

  //TEST DATA
  tags: Tag[] = [
    {name: 'Aussichtspunkt'},
    {name: 'Baum'},
    {name: 'Statue'}
  ]

  constructor() { }

  ngOnInit(): void {
  }

}
