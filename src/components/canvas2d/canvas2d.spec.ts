import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Canvas2d } from './canvas2d';

describe('Canvas2d', () => {
  let component: Canvas2d;
  let fixture: ComponentFixture<Canvas2d>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Canvas2d]
    })
    .compileComponents();

    fixture = TestBed.createComponent(Canvas2d);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
