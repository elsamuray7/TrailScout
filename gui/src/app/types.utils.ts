import { NgbTimeStruct } from "@ng-bootstrap/ng-bootstrap";
import { Sight } from "./data/Sight";

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

export interface RouteTrackerSection {
    section: L.LatLng[];
    sight: Sight;
    routeId: number;
  }