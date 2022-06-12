import { Component, Input, OnInit, Output, EventEmitter } from '@angular/core';
import { Sight, TagCheckboxResponse } from 'src/app/types.utils';

@Component({
  selector: 'app-settings-taskbar-tag-item',
  templateUrl: './settings-taskbar-tag-item.component.html',
  styleUrls: ['./settings-taskbar-tag-item.component.scss']
})
export class SettingsTaskbarTagItemComponent implements OnInit {

  @Input() sight!: Sight;
  @Output('checked') checkedEvent = new EventEmitter;

  checked = false;
  prio: number = 0;
  constructor() { }

  ngOnInit(): void {
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

}
