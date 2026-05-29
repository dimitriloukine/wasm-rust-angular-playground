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
  frameTimes120 = signal<number[]>([]);
  frameRate = computed(() => Math.floor(1000 / this.frameTime()));
  averageFrameTime = computed(() => {
    const times = this.frameTimes120();
    if (times.length === 0) return 0;
    return times.reduce((sum, time) => sum + time, 0) / times.length;
  });
  averageFrameRate = computed(() => Math.floor(1000 / this.averageFrameTime()));
  peakFrameTime = computed(() => {
    const times = this.frameTimes120();
    return times.length > 0 ? Math.max(...times) : 0;
  });
  floorFrameTime = computed(() => {
    const times = this.frameTimes120();
    return times.length > 0 ? Math.min(...times) : 0;
  });

  ngOnInit(): void {
    this.setupCanvas2DRenderer();
  }

  ngOnDestroy(): void {
    if (this.animationFrameId !== undefined) {
      cancelAnimationFrame(this.animationFrameId);
    }
  }

  private loadTexture(path: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = reject;
      img.src = path;
    });
  }

  private async setupCanvas2DRenderer(): Promise<void> {
    const canvas = this.canvasRef.nativeElement;
    const ctx = canvas.getContext('2d')!;

    // Load WASM module
    const wasm = await this.wasmLoader.loadModule('/wasm-canvas/wasm_canvas.js');

    // Try to load texture from PNG file, fallback to procedural if it fails
    let renderer;
    const textureImage = await this.loadTexture('/dirt-1-128.png');
    const textureSize = 128;

    // Extract RGBA pixel data from image
    const tempCanvas = document.createElement('canvas');
    tempCanvas.width = textureSize;
    tempCanvas.height = textureSize;
    const tempCtx = tempCanvas.getContext('2d')!;
    tempCtx.drawImage(textureImage, 0, 0);
    const imageData = tempCtx.getImageData(0, 0, textureSize, textureSize);
    const texturePixels = Array.from(imageData.data);

    console.log('✅ Loaded PNG texture:', textureSize, 'x', textureSize, 'pixels');
    renderer = wasm.SoftwareRenderer.new_with_texture(640, 480, textureSize, texturePixels);

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
      const currentFrameTime = renderer.frame_time();
      this.frameTime.set(currentFrameTime);

      // Maintain rolling buffer of last 120 frame times
      const times = [...this.frameTimes120(), currentFrameTime];
      this.frameTimes120.set(times.slice(-120));

      // Get pixel data from Rust and display directly
      const pixels = renderer.get_pixels();
      const imageData = new ImageData(new Uint8ClampedArray(pixels), 640, 480);
      ctx.putImageData(imageData, 0, 0);

      this.animationFrameId = requestAnimationFrame(animate);
    };

    this.animationFrameId = requestAnimationFrame(animate);
  }
}
