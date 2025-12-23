import { Component, OnInit, ChangeDetectorRef, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WasmLoaderService } from '../../services/wasm-loader.service';

@Component({
  selector: 'app-basic',
  imports: [CommonModule],
  templateUrl: './basic.html',
  styleUrl: './basic.scss',
})
export class Basic implements OnInit {
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly wasmLoader = inject(WasmLoaderService);

  greetingMessage = 'Loading WASM...';
  additionResult = '';

  ngOnInit(): void {
    this.loadWasm();
  }

  private async loadWasm(): Promise<void> {
    // Load the WASM module
    const wasm = await this.wasmLoader.loadModule('/wasm-hello/wasm_hello.js');

    // Call Rust functions
    this.greetingMessage = wasm.greet('Angular Developer');
    const sum = wasm.add(42, 58);
    this.additionResult = `42 + 58 = ${sum} (calculated in Rust/WASM!)`;

    // Update the view
    this.cdr.detectChanges();
  }
}
