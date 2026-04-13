import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Next } from './next';

describe('Next', () => {
  let component: Next;
  let fixture: ComponentFixture<Next>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Next]
    })
    .compileComponents();

    fixture = TestBed.createComponent(Next);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
