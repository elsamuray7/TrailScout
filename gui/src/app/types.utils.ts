import { NgbTimeStruct } from "@ng-bootstrap/ng-bootstrap";

export interface Settings {
    radius?: number;
    startTime?: NgbTimeStruct;
    walkTime?: NgbTimeStruct;
    endTime?: NgbTimeStruct;
}

export interface Cookie {
    key: string;
    value: any;
}
