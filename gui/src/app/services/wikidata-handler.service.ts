import { HttpClient } from '@angular/common/http';
import { Injectable, TemplateRef } from '@angular/core';


@Injectable({ providedIn: 'root' })
export class WikidataHandlerService {

    constructor(private http:  HttpClient) {}

    getWiki(id: string) {
        const baseUrl = 'http://www.wikidata.org/w/api.php';
        if (id === 'empty') {
          return;
        }
        return this.http.get(baseUrl+ '?action=wbgetentities&format=json&ids=' + id + "&origin=*").toPromise();
      }
      
      
}
