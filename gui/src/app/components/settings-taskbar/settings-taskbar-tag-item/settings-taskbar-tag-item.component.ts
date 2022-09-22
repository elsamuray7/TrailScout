import { Component, Input, OnInit, Output, EventEmitter, ViewChild } from '@angular/core';
import { NgbTypeahead } from '@ng-bootstrap/ng-bootstrap';
import {Category} from "../../../data/Category";
import { BehaviorSubject, debounceTime, distinctUntilChanged, filter, map, merge, Observable, OperatorFunction, Subject } from 'rxjs';
import { Sight } from '../../../data/Sight';

@Component({
  selector: 'app-settings-taskbar-tag-item',
  templateUrl: './settings-taskbar-tag-item.component.html',
  styleUrls: ['./settings-taskbar-tag-item.component.scss']
})
export class SettingsTaskbarTagItemComponent implements OnInit {

  @Input() category!: Category;
  @Output('checked') checkedEvent = new EventEmitter;
  @Output() showSightEvent = new EventEmitter<Sight>();

  @ViewChild('sightSearch', {static: true}) sightSearch: NgbTypeahead;
  focus$ = new Subject<string>();
  click$ = new Subject<string>();
  public model: String;
  sightsWithSpecialPref: BehaviorSubject<Sight[]> = new BehaviorSubject<Sight[]>([]);

  checked = false;
  prio: number = 3;
  readonly priorityLabels = new Map<number, string>([
    [0, "Gar Nicht"],
    [1, "Sehr Niedrig"],
    [2, "Niedrig"],
    [3, "Neutral"],
    [4, "Hoch"],
    [5, "Sehr Hoch"]
  ]);

  readonly categoryLabels = new Map<string, string>([
    ["Sightseeing", "Sehenswürdigkeiten"],
    ["Activities", "Aktivitäten"],
    ["Nightlife", "Nachtleben"],
    ["Restaurants", "Restaurants"],
    ["Shopping", "Shopping"],
    ["PicnicBarbequeSpot", "Picknick & Grillen"],
    ["MuseumExhibition", "Museen"],
    ["Nature", "Natur"],
    ["Swimming", "Badeplätze"],
    ["Animals", "Tiere"]
  ]);

  constructor() {
   }

  ngOnInit(): void {
    this.sightsWithSpecialPref.next(this.category.getAllSightsWithSpecialPref());
    this.sightSearch.selectItem.subscribe((item) => {
      var sight = item.item as Sight;
      sight.pref = this.category.pref;
      this.sightsWithSpecialPref.next(this.category.getAllSightsWithSpecialPref());
    });
  }

  removeSpecialPrefSight(sight: Sight) {
    sight.pref = -1;
    this.sightsWithSpecialPref.next(this.category.getAllSightsWithSpecialPref());
  }

  showSight(sight: Sight) {
    this.showSightEvent.emit(sight);
  }

  checkedTag() {
    this.checked = !this.checked;
    this.checkedEvent.emit(this.checked);
  }

  formatter = (sight: Sight) => sight.name;

  search: OperatorFunction<string, readonly Sight[]> = (text$: Observable<string>) => {
    const debouncedText$ = text$.pipe(debounceTime(200), distinctUntilChanged());
    const clicksWithClosedPopup$ = this.click$.pipe(filter(() => !this.sightSearch.isPopupOpen()));
    const inputFocus$ = this.focus$;

    return merge(debouncedText$, inputFocus$, clicksWithClosedPopup$).pipe(
      map(term => (term === '' ? this.category.sights
        : this.category.sights.filter(v => v.name.toLowerCase().indexOf(term.toLowerCase()) > -1)).slice(0, 10))
    );
  };

  getImage() {
    return {'background-image' : 'url(assets/sights/' + this.category.name + '.jpg)' };
  }
}
