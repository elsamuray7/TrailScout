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
    ["PicnicBarbequeSpot", "Picnic Plätze"],
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
    //const response: TagCheckboxResponse = {checked: this.checked, sight: this.sight, prio: 0};
    //this.checkedEvent.emit(response);
  }

  prioChanged() {
    //const response: TagCheckboxResponse = {checked: this.checked, sight: this.sight, prio: this.prio};
    //this.checkedEvent.emit(response);
  }

  getImage() {
    return {'background-image' : 'url(assets/sights/' + this.category.name + '.jpg)' };
  }
}
