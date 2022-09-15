import { HttpClient } from '@angular/common/http';
import { Injectable, TemplateRef } from '@angular/core';

interface EntityData {
  aliases: any;
  claims: {
    P18: [{
      mainsnak: {
        datavalue: {
          value: string
        }
      }
    }]
  };
  descriptions: any;
  id: string;
  labels: any;
  siteLinks: any;
}

interface Entity {
  [key: string]: EntityData;
}

export interface WikiResult {
  entities: Entity;
  success: number;
}

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
      
    getImagePath(image: string) {
      const baseUrl = 'https://commons.wikimedia.org/w/index.php?title=Special:Redirect/file/';
      return baseUrl + image;
    }
      
}
