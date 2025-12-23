import { Routes } from '@angular/router';
import { Basic } from './components/basic/basic';
import { Canvas } from '../components/canvas/canvas';

export const routes: Routes = [
  { title: 'Basic', path: 'basic', component: Basic },
  {
    title: 'Canvas',
    path: 'canvas',
    component: Canvas,
  },
];
