# WASM Hello World Implementation

## 🦀 Rust Setup

- Created new Rust library: `cargo new --lib wasm-hello`
- Configured `Cargo.toml` with:
  - `crate-type = ["cdylib"]` for WASM compilation
  - `wasm-bindgen = "0.2"` dependency
- Implemented two functions in `lib.rs`:
  - `get_hello_world()` - returns simple hello message
  - `greet(name)` - returns personalized greeting

## 📦 WASM Build

- Built module using npm script: `npm run wasm:hello`
- Build process:
  - Compiles Rust to WebAssembly
  - Generates JavaScript bindings
  - Copies output to `public/wasm-hello/`
- Generated files:
  - `wasm_hello.js` - JavaScript bindings
  - `wasm_hello_bg.wasm` - Compiled binary
  - `wasm_hello.d.ts` - TypeScript definitions

## 🅰️ Angular Integration

### Component Setup

- Injected `WasmLoaderService` for module loading
- Used Angular signals for reactive state
- Implemented `ngOnInit()` lifecycle hook

### WASM Loading

- Loaded module: `loadModule('/wasm-hello/wasm_hello.js')`
- Called Rust functions from JavaScript:
  - `wasm.get_hello_world()` → main message
  - `wasm.greet('Angular Developer')` → personalized greeting
- Error handling with try/catch block

### Template

- Two message boxes displaying:
  - Hello World message from Rust
  - Personalized greeting with user name
- Conditional rendering with `*ngIf`
- Clean, styled layout with SCSS

## ✅ Result

- Rust code compiles to WebAssembly
- Angular seamlessly calls Rust functions
- Messages display in real-time
- Full TypeScript type safety
- Reusable pattern for future WASM modules
