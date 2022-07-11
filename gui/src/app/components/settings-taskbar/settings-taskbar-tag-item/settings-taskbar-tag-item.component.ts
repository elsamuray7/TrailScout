import { Component, Input, OnInit, Output, EventEmitter } from '@angular/core';
import {Category} from "../../../data/Category";

@Component({
  selector: 'app-settings-taskbar-tag-item',
  templateUrl: './settings-taskbar-tag-item.component.html',
  styleUrls: ['./settings-taskbar-tag-item.component.scss']
})
export class SettingsTaskbarTagItemComponent implements OnInit {

  @Input() category!: Category;
  @Output('checked') checkedEvent = new EventEmitter;

  checked = false;
  prio: number = 3;
  readonly priorityLabels = new Map<number, string>([
    [0, "Gar Nicht"],
    [1, "Niedriger"],
    [2, "Niedrig"],
    [3, "Neutral"],
    [4, "Hoch"],
    [5, "Höher"]
  ]);

  readonly categoryLabels = new Map<string, string>([
    ["Sightseeing", "Sehenswürdigkeiten"],
    ["Other", "Andere"],
    ["Nightlife", "Nachtleben"],
    ["Restaurants", "Restaurants"],
    ["Shopping", "Shopping"],
    ["PicnicBarbequeSpot", "Picknick & Grillen"],
    ["MuseumExhibition", "Museen"],
    ["Nature", "Natur"],
    ["Swimming", "Badeplätze"]
  ]);

  constructor() {
   }

  ngOnInit(): void {
  }

  checkedTag() {
    this.checked = !this.checked;
    this.checkedEvent.emit(this.checked);
  }

  getImage() {
    //TODO: fix - rename images to fit the category names
    return {'background-image' : 'url(assets/sights/' + this.category.name + '.jpg)' };
  }
}
