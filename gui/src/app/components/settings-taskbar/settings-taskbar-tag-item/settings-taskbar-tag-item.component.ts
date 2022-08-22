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

  @ViewChild('sightSearch', {static: true}) sightSearch: NgbTypeahead;
  focus$ = new Subject<string>();
  click$ = new Subject<string>();
  public model: String;
  sightsWithSpecialPref: BehaviorSubject<Sight[]> = new BehaviorSubject<Sight[]>([]);

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
    this.sightsWithSpecialPref.next(this.category.getAllSightsWithSpecialPref());
    this.sightSearch.selectItem.subscribe((item) => {
      console.log(item);
      var sight = item.item as Sight;
      sight.pref = this.category.pref;
      console.log(this.category.getAllSightsWithSpecialPref());
      this.sightsWithSpecialPref.next(this.category.getAllSightsWithSpecialPref());
    });
  }

  removeSpecialPrefSight(sight: Sight) {
    sight.pref = -1;
    console.log("removing: " + sight);
    this.sightsWithSpecialPref.next(this.category.getAllSightsWithSpecialPref());
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

  addSingleSight(value: Event) {
    console.log(value);
  }
}
