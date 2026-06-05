import { Routes } from '@angular/router';
import { Canvas2d } from './components/canvas2d/canvas2d';
import { Next } from './components/next/next';
import { Hello } from './components/hello/hello';

export const routes: Routes = [
  {
    title: 'Canvas 2D',
    path: 'canvas2d',
    component: Canvas2d,
  },
  {
    title: 'Next',
    path: 'next',
    component: Next,
  },
  {
    title: 'hello',
    path: 'hello',
    component: Hello,
  },
];
