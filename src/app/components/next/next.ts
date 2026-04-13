import {
  ChangeDetectorRef,
  Component,
  ElementRef,
  inject,
  OnInit,
  signal,
  ViewChild,
} from '@angular/core';
import { WasmLoaderService } from '@app/services/wasm-loader.service';

@Component({
  selector: 'app-next',
  imports: [],
  templateUrl: './next.html',
  styleUrl: './next.scss',
})
export class Next implements OnInit {
  @ViewChild('canvas', { static: true }) canvasRef!: ElementRef<HTMLCanvasElement>;
  wasmLoader = inject(WasmLoaderService);
  cdr = inject(ChangeDetectorRef);
  sum = signal(0);

  ngOnInit(): void {
    this.loadWasm();
  }

  private async loadWasm(): Promise<void> {
    const wasm = await this.wasmLoader.loadModule('/wasm-next/wasm_next.js');

    this.sum.set(wasm.add(42, 58));

    const renderer = wasm.Renderer.new(640, 480);

    this.cdr.detectChanges();

    const animate = () => {
      const canvas = this.canvasRef.nativeElement;
      const ctx = canvas.getContext('2d')!;
      const pixels = renderer.get_frame_buffer();
      const imageData = new ImageData(new Uint8ClampedArray(pixels), 640, 480);
      ctx.putImageData(imageData, 0, 0);
      requestAnimationFrame(animate);
    };
    requestAnimationFrame(animate);
  }
}
