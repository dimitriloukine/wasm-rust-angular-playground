# Rust/WASM Setup Guide for Angular

This document outlines the steps taken to integrate Rust-compiled WebAssembly into an Angular application.

## Prerequisites Installation

### 1. Install Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 2. Install wasm-pack

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### 3. Add WASM Target to Rust

```bash
rustup target add wasm32-unknown-unknown
```

## Rust Project Setup

### 1. Create Rust Library Project

```bash
cd /path/to/angular/project
cargo new --lib wasm-hello
```

### 2. Configure Cargo.toml

Update `wasm-hello/Cargo.toml`:

```toml
[package]
name = "wasm-hello"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
```

### 3. Implement Rust Functions

Update `wasm-hello/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! This message comes from Rust/WASM!", name)
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 4. Build WASM Module

```bash
cd wasm-hello
wasm-pack build --target web
```

This generates a `pkg/` directory with:

- `wasm_hello.js` - JavaScript bindings
- `wasm_hello_bg.wasm` - Compiled WebAssembly binary
- `wasm_hello.d.ts` - TypeScript definitions

## Angular Integration

### 1. Copy WASM Files to Public Directory

```bash
cp -r wasm-hello/pkg public/wasm-hello
```

This makes the WASM files accessible to the browser at runtime.

### 2. Update Angular Component

**Component TypeScript** (`basic.ts`):

```typescript
import { Component, OnInit, ChangeDetectorRef, inject, PLATFORM_ID } from '@angular/core';
import { CommonModule, DOCUMENT, isPlatformBrowser } from '@angular/common';

@Component({
  selector: 'app-basic',
  imports: [CommonModule],
  templateUrl: './basic.html',
  styleUrl: './basic.scss',
})
export class Basic implements OnInit {
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly document = inject(DOCUMENT);
  private readonly platformId = inject(PLATFORM_ID);
  greetingMessage: string = 'Loading WASM...';
  additionResult: string = '';

  ngOnInit(): void {
    if (isPlatformBrowser(this.platformId)) {
      this.loadWasm();
    }
  }

  private async loadWasm(): Promise<void> {
    try {
      const window = this.document.defaultView;
      if (!window) {
        throw new Error('Window is not available');
      }

      // Load the WASM JavaScript wrapper using a script tag approach
      const script = this.document.createElement('script');
      script.type = 'module';
      script.textContent = `
        import init, { greet, add } from '/wasm-hello/wasm_hello.js';
        await init();
        window.__wasmModule = { greet, add };
      `;
      this.document.head.appendChild(script);

      // Wait for the module to load
      await new Promise((resolve) => {
        const checkLoaded = setInterval(() => {
          if ((window as any).__wasmModule) {
            clearInterval(checkLoaded);
            resolve(true);
          }
        }, 50);
      });

      const wasmModule = (window as any).__wasmModule;

      // Call the greet function from Rust
      this.greetingMessage = wasmModule.greet('Angular Developer');

      // Call the add function from Rust
      const sum = wasmModule.add(42, 58);
      this.additionResult = `42 + 58 = ${sum} (calculated in Rust/WASM!)`;

      // Trigger change detection
      this.cdr.detectChanges();
    } catch (error) {
      console.error('Failed to load WASM module:', error);
      this.greetingMessage = 'Failed to load WASM module';
      this.cdr.detectChanges();
    }
  }
}
```

**Component Template** (`basic.html`):

```html
<div class="wasm-demo">
  <h2>Rust/WASM Hello World Demo</h2>

  <div class="result">
    <h3>Greeting from Rust:</h3>
    <p>{{ greetingMessage }}</p>
  </div>

  <div class="result" *ngIf="additionResult">
    <h3>Math from Rust:</h3>
    <p>{{ additionResult }}</p>
  </div>
</div>
```

### 3. Create TypeScript Declarations (Optional)

Create `src/wasm.d.ts` for better IDE support:

```typescript
declare module '/wasm-hello/wasm_hello.js' {
  export default function init(): Promise<void>;
  export function greet(name: string): string;
  export function add(a: number, b: number): number;
}
```

## Key Implementation Details

### Why Script Tag Approach?

We use a dynamic script tag instead of direct `import()` because:

- Vite's import analysis tries to resolve imports at build time
- WASM files in `public/` directory are only available at runtime
- Script tags with `type="module"` bypass Vite's bundling process
- The `/* @vite-ignore */` comment didn't work reliably with this setup

### Angular Dependency Injection

The implementation uses Angular's DI system properly:

- `DOCUMENT` token to access DOM (SSR-safe)
- `PLATFORM_ID` to check browser environment
- `ChangeDetectorRef` to manually trigger UI updates after async WASM loading

### Change Detection

`this.cdr.detectChanges()` is required because:

- WASM loading happens asynchronously outside Angular's zone
- Angular doesn't automatically detect changes from dynamic script execution
- Manual trigger ensures the UI updates when WASM functions return values

## Development Workflow

### Modify Rust Code

1. Edit `wasm-hello/src/lib.rs`
2. Rebuild: `cd wasm-hello && wasm-pack build --target web`
3. Copy to public: `cp -r pkg ../public/wasm-hello`
4. Angular dev server will auto-reload

### Quick Rebuild Script

Create a helper script for faster iteration:

```bash
#!/bin/bash
cd wasm-hello
wasm-pack build --target web
cp -r pkg ../public/wasm-hello
echo "WASM module rebuilt and copied!"
```

## Troubleshooting

### WASM 404 Errors

- Ensure files are in `public/wasm-hello/` directory
- Check browser DevTools Network tab for actual request path
- Verify Angular dev server is serving files from `public/`

### TypeScript Errors

- Add `@ts-ignore` comment if needed for dynamic imports
- Ensure `src/wasm.d.ts` is included in `tsconfig.json`

### Change Detection Not Working

- Ensure you're using `inject(ChangeDetectorRef)`
- Call `this.cdr.detectChanges()` after WASM operations

## Benefits of This Approach

1. **Performance**: Rust compiles to highly optimized WASM
2. **Type Safety**: Strong typing in both Rust and TypeScript
3. **SSR Compatible**: Platform checks prevent server-side errors
4. **Clean Separation**: Rust logic isolated in separate library
5. **Angular Best Practices**: Uses DI, change detection, and platform checks properly

## Next Steps

- Add more complex Rust functions (data processing, algorithms)
- Implement bidirectional data passing between JS and Rust
- Explore Rust's memory management benefits for large datasets
- Consider using `wasm-bindgen-futures` for async Rust code
