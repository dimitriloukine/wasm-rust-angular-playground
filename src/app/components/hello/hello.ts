import { Component, EnvironmentInjector, OnInit, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WasmLoaderService } from '@app/services/wasm-loader.service';

@Component({
  selector: 'app-hello',
  imports: [CommonModule],
  templateUrl: './hello.html',
  styleUrl: './hello.scss',
})
export class Hello implements OnInit {
  private readonly wasmLoader = inject(WasmLoaderService);

  helloMessage = signal<string>('Loading WASM...');
  greetMessage = signal<string>('');

  async ngOnInit(): Promise<void> {
    await this.loadWasmModule();
  }

  private async loadWasmModule(): Promise<void> {
    try {
      const wasm = await this.wasmLoader.loadModule('/wasm-hello/wasm_hello.js');

      // Call the hello world function
      this.helloMessage.set(wasm.get_hello_world());

      // Call the greet function with a name
      this.greetMessage.set(wasm.greet('Front Academy'));
    } catch (error) {
      console.error('Failed to load WASM module:', error);
      this.helloMessage.set('Failed to load WASM module');
    }
  }
}
