import { NgbTimeStruct } from "@ng-bootstrap/ng-bootstrap";



export interface TagCheckboxResponse {
    sight: Sight;
    checked: boolean;
    prio: number;
}

export interface Settings {
    radius?: number;
    startTime?: NgbTimeStruct;
    walkTime?: NgbTimeStruct;
    endTime?: NgbTimeStruct;
    sights: Map<string, number>;
}

export interface SightsPrios {
    sightWithPrio: Map<string, number>;
}

export interface Sight {
    id: string;
    name: string;
    description?: string;
}