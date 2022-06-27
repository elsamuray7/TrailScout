import { TestBed } from '@angular/core/testing';

import { SightsServiceService } from './sights-service.service';

describe('SightsServiceService', () => {
  let service: SightsServiceService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(SightsServiceService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
