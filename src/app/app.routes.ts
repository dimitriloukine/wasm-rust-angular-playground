import { Routes } from '@angular/router';
import { Basic } from './components/basic/basic';
import { Canvas } from '../components/canvas/canvas';
import { Canvas2d } from '../components/canvas2d/canvas2d';

export const routes: Routes = [
  { title: 'Basic', path: 'basic', component: Basic },
  {
    title: 'Canvas',
    path: 'canvas',
    component: Canvas,
  },
  {
    title: 'Canvas 2D',
    path: 'canvas2d',
    component: Canvas2d,
  },
];
