import { NgbTimeStruct } from "@ng-bootstrap/ng-bootstrap";

export interface Tag {
    name: string;
}

export interface TagCheckboxResponse {
    tag: Tag;
    checked: boolean;
}

export interface Settings {
    radius?: number;
    startTime?: NgbTimeStruct;
    walkTime?: NgbTimeStruct;
    endTime?: NgbTimeStruct;
    tags: Tag[];
}