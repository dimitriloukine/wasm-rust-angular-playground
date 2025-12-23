import { Injectable, inject } from '@angular/core';
import { DOCUMENT } from '@angular/common';

@Injectable({
  providedIn: 'root',
})
export class WasmLoaderService {
  private readonly document = inject(DOCUMENT);

  async loadModule(modulePath: string): Promise<any> {
    const window = this.document.defaultView!;
    const moduleKey = `__wasm_${modulePath.replace(/[^a-zA-Z0-9]/g, '_')}`;

    // Create and inject script tag
    const script = this.document.createElement('script');
    script.type = 'module';
    script.textContent = `
      import init, * as wasmModule from '${modulePath}';
      await init();
      window.${moduleKey} = wasmModule;
    `;
    this.document.head.appendChild(script);

    // Wait for module to be available on window
    await new Promise((resolve) => {
      const checkLoaded = setInterval(() => {
        if ((window as any)[moduleKey]) {
          clearInterval(checkLoaded);
          resolve(true);
        }
      }, 50);
    });

    return (window as any)[moduleKey];
  }
}
