// Import wasm_bindgen types and macros - Docs: https://rustwasm.github.io/wasm-bindgen/
use wasm_bindgen::prelude::*;

// Struct to hold our software renderer state - Docs: https://doc.rust-lang.org/book/ch05-00-structs.html
#[wasm_bindgen]
pub struct SoftwareRenderer {
    pixels: Vec<u8>,  // Our pixel buffer (RGBA format)
    width: u32,
    height: u32,
    square_size: u32,
}

// Implementation block - methods for our struct - Docs: https://doc.rust-lang.org/book/ch05-03-method-syntax.html
#[wasm_bindgen]
impl SoftwareRenderer {
    // Constructor - called from JS with SoftwareRenderer.new() - Docs: https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-rust-exports.html
    pub fn new(width: u32, height: u32, square_size: u32) -> Self {
        let pixels = Vec::with_capacity((width * height * 4) as usize);
        Self { pixels, width, height, square_size }  // Self = SoftwareRenderer type
    }
    
    // Render a frame - updates the pixel buffer
    pub fn render_frame(&mut self) {  // &mut self = mutable reference to this instance - Docs: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
        self.pixels.clear();  // Clear previous frame
        
        for y in 0..self.height {
            for x in 0..self.width {
                let square_x = x / self.square_size;
                let square_y = y / self.square_size;
                let is_red = (square_x + square_y) % 2 == 0;
                
                if is_red {
                    self.pixels.push(255); // R
                    self.pixels.push(0);   // G
                    self.pixels.push(0);   // B
                    self.pixels.push(255); // A
                } else {
                    self.pixels.push(255); // R
                    self.pixels.push(255); // G
                    self.pixels.push(255); // B
                    self.pixels.push(255); // A
                }
            }
        }
    }
    
    // Get pointer to pixel data for WebGL upload - Docs: https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer
    pub fn get_pixel_ptr(&self) -> *const u8 {  // *const u8 = raw pointer (for JS to access WASM memory)
        self.pixels.as_ptr()
    }
    
    // Get pixel data as a JS-accessible slice
    pub fn get_pixels(&self) -> Vec<u8> {
        self.pixels.clone()
    }
    
    // Getters for dimensions
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
}
