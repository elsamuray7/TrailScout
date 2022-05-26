import { Component, Input, OnInit, Output, EventEmitter } from '@angular/core';
import { Tag, TagCheckboxResponse } from 'src/app/types.utils';

@Component({
  selector: 'app-settings-taskbar-tag-item',
  templateUrl: './settings-taskbar-tag-item.component.html',
  styleUrls: ['./settings-taskbar-tag-item.component.scss']
})
export class SettingsTaskbarTagItemComponent implements OnInit {

  @Input() tag!: Tag;
  @Output() checked = new EventEmitter;
  constructor() { }

  ngOnInit(): void {
  }

  checkedTag(tag: Tag, checked: any) {
    const response: TagCheckboxResponse = {checked: checked, tag: tag}
    this.checked.emit(response);
  }

}
