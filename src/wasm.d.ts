declare module '/wasm-hello/wasm_hello.js' {
  export default function init(): Promise<void>;
  export function greet(name: string): string;
  export function add(a: number, b: number): number;
}
