import { Component, Input, OnInit, Output, EventEmitter } from '@angular/core';
import { Tag, TagCheckboxResponse } from 'src/app/types.utils';

@Component({
  selector: 'app-settings-taskbar-tag-item',
  templateUrl: './settings-taskbar-tag-item.component.html',
  styleUrls: ['./settings-taskbar-tag-item.component.scss']
})
export class SettingsTaskbarTagItemComponent implements OnInit {

  @Input() tag!: Tag;
  @Output('checked') checkedEvent = new EventEmitter;

  checked = false;
  prio: number = 0;
  constructor() { }

  ngOnInit(): void {
  }

  checkedTag(tag: Tag, checked: any) {
    this.checked = checked;
    const response: TagCheckboxResponse = {checked: checked, tag: tag}
    this.checkedEvent.emit(response);
  }

}
