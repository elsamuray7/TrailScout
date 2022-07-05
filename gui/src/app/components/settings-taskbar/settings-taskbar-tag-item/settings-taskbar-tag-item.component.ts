import { Component, Input, OnInit, Output, EventEmitter } from '@angular/core';
import { Sight, TagCheckboxResponse } from 'src/app/types.utils';

@Component({
  selector: 'app-settings-taskbar-tag-item',
  templateUrl: './settings-taskbar-tag-item.component.html',
  styleUrls: ['./settings-taskbar-tag-item.component.scss']
})
export class SettingsTaskbarTagItemComponent implements OnInit {

  @Input() sight!: Sight;
  @Input() cookiePrio?: number
  @Output('checked') checkedEvent = new EventEmitter;

  imagePath?: string;
  checked = false;
  prio: number = 3;
  readonly priorityLabels = new Map<number, string>([
    [0, "Gar Nicht"],
    [1, "Niedriger"],
    [2, "Niedrig"],
    [3, "Neutral"],
    [4, "Hoch"],
    [5, "Höher"]
  ])
  constructor() {
   }

  ngOnInit(): void {
    if (this.cookiePrio) {
      this.prio = this.cookiePrio;
      this.checked = true;
      this.prioChanged();
    }
    this.imagePath = this.sight.imagePath;
  }

  checkedTag() {
    this.checked = !this.checked;
    const response: TagCheckboxResponse = {checked: this.checked, sight: this.sight, prio: 0};
    this.checkedEvent.emit(response);
  }

  prioChanged() {
    const response: TagCheckboxResponse = {checked: this.checked, sight: this.sight, prio: this.prio};
    this.checkedEvent.emit(response);
  }

  getImage() {
    return {'background-image' : 'url(assets/sights/' + this.imagePath + ')' };
  }
}
