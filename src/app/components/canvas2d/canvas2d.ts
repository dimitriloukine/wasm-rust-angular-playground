import {
  Component,
  OnInit,
  OnDestroy,
  ViewChild,
  ElementRef,
  inject,
  signal,
  computed,
  HostListener,
} from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { WasmLoaderService } from '@app/services/wasm-loader.service';

@Component({
  selector: 'app-canvas2d',
  imports: [DecimalPipe],
  templateUrl: './canvas2d.html',
  styleUrl: './canvas2d.scss',
})
export class Canvas2d implements OnInit, OnDestroy {
  @ViewChild('canvas', { static: true }) canvasRef!: ElementRef<HTMLCanvasElement>;

  private readonly wasmLoader = inject(WasmLoaderService);
  private animationFrameId?: number;

  // Track keyboard input
  private readonly keysPressed = new Set<string>();

  @HostListener('window:keydown', ['$event'])
  onKeyDown(event: KeyboardEvent) {
    this.keysPressed.add(event.key);
  }

  @HostListener('window:keyup', ['$event'])
  onKeyUp(event: KeyboardEvent) {
    this.keysPressed.delete(event.key);
  }

  frameTime = signal(1);
  frameRate = computed(() => Math.floor(1000 / this.frameTime()));

  ngOnInit(): void {
    this.setupCanvas2DRenderer();
  }

  ngOnDestroy(): void {
    if (this.animationFrameId !== undefined) {
      cancelAnimationFrame(this.animationFrameId);
    }
  }

  private async setupCanvas2DRenderer(): Promise<void> {
    const canvas = this.canvasRef.nativeElement;
    const ctx = canvas.getContext('2d')!;

    // Load WASM module
    const wasm = await this.wasmLoader.loadModule('/wasm-canvas/wasm_canvas.js');

    // Create renderer instance in Rust
    const renderer = wasm.SoftwareRenderer.new(640, 480, 32);

    // Render loop with delta time tracking
    let lastTime = 0;
    const animate = (timestamp: number) => {
      // Initialize lastTime on first frame
      if (lastTime === 0) {
        lastTime = timestamp;
      }

      const deltaTime = Math.floor(timestamp - lastTime);
      this.frameTime.set(deltaTime);
      lastTime = timestamp;

      // Update animation in Rust
      const keys = Array.from(this.keysPressed);
      renderer.update(deltaTime, keys);

      // Rust generates pixels
      renderer.render_frame();

      // Get frame time from Rust (it stores the delta we passed)
      this.frameTime.set(renderer.frame_time());

      // Get pixel data from Rust and display directly
      const pixels = renderer.get_pixels();
      const imageData = new ImageData(new Uint8ClampedArray(pixels), 640, 480);
      ctx.putImageData(imageData, 0, 0);

      this.animationFrameId = requestAnimationFrame(animate);
    };

    this.animationFrameId = requestAnimationFrame(animate);
  }
}
