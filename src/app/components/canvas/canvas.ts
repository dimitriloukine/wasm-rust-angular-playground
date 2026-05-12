import {
  Component,
  OnInit,
  OnDestroy,
  ViewChild,
  ElementRef,
  inject,
  HostListener,
  computed,
  signal,
} from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { WasmLoaderService } from '@app/services/wasm-loader.service';

@Component({
  selector: 'app-canvas',
  imports: [DecimalPipe],
  templateUrl: './canvas.html',
  styleUrl: './canvas.scss',
})
export class Canvas implements OnInit, OnDestroy {
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
    this.setupWebGLRenderer();
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

  private async setupWebGLRenderer(): Promise<void> {
    const canvas = this.canvasRef.nativeElement;
    const gl = canvas.getContext('webgl')!;

    // Load WASM module
    const wasm = await this.wasmLoader.loadModule('/wasm-canvas/wasm_canvas.js');

    // Try to load texture from PNG file, fallback to procedural if it fails
    let renderer;
    try {
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
    } catch (error) {
      console.warn('⚠️ Failed to load texture, using procedural fallback:', error);
      renderer = wasm.SoftwareRenderer.new(640, 480, 32);
    }

    // Setup WebGL texture
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // Create vertex shader (simple pass-through)
    const vertexShader = gl.createShader(gl.VERTEX_SHADER)!;
    gl.shaderSource(
      vertexShader,
      `
      attribute vec2 position;
      varying vec2 texCoord;
      void main() {
        texCoord = vec2(position.x * 0.5 + 0.5, 1.0 - (position.y * 0.5 + 0.5));
        gl_Position = vec4(position, 0.0, 1.0);
      }
    `,
    );
    gl.compileShader(vertexShader);

    // Create fragment shader (samples texture)
    const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER)!;
    gl.shaderSource(
      fragmentShader,
      `
      precision mediump float;
      varying vec2 texCoord;
      uniform sampler2D texture;
      void main() {
        gl_FragColor = texture2D(texture, texCoord);
      }
    `,
    );
    gl.compileShader(fragmentShader);

    // Link shader program
    const program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);
    gl.useProgram(program);

    // Setup full-screen quad
    const positions = new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

    const positionLoc = gl.getAttribLocation(program, 'position');
    gl.enableVertexAttribArray(positionLoc);
    gl.vertexAttribPointer(positionLoc, 2, gl.FLOAT, false, 0, 0);

    // Render loop with delta time tracking
    let lastTime = 0;
    const animate = (timestamp: number) => {
      // Initialize lastTime on first frame
      if (lastTime === 0) {
        lastTime = timestamp;
      }

      const deltaTime = Math.max(1, Math.floor(timestamp - lastTime)); // Ensure minimum 1ms to avoid zero delta

      lastTime = timestamp;

      // Pass pressed keys to Rust - it decides what to do with them
      const keys = Array.from(this.keysPressed);
      renderer.update(deltaTime, keys);

      // Rust generates pixels
      renderer.render_frame();

      // Get frame time from Rust (it stores the delta we passed)
      this.frameTime.set(renderer.frame_time());

      // Get pixel data from Rust
      const pixels = renderer.get_pixels();

      // Upload to GPU texture
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 640, 480, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

      // Clear and draw
      gl.clearColor(0, 0, 0, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 6);

      this.animationFrameId = requestAnimationFrame(animate);
    };

    this.animationFrameId = requestAnimationFrame(animate);
  }
}
