use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

#[wasm_bindgen]
pub struct Renderer {
    frame_buffer: Vec<u32>, // framebuffer, pixels are ABGR u8 packed into a u32;
}

#[wasm_bindgen]
impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count: usize = (width * height) as usize; // we cast using as usize because this will be used as a Vector index, .try_into().unwrap() adds a runtime check that we do not need for WASM
        let pixels: Vec<u32> = vec![0xFF_FF_00_00; pixel_count]; // ABGR
        Self {
            frame_buffer: pixels,
        }
    }

    pub fn get_frame_buffer(&self) -> Vec<u8> {
        // repack the pixels as u8
        self.frame_buffer
            .iter() // iterate over each u32 pixel
            .flat_map(|&pixel| pixel.to_le_bytes()) // converts u32 to [u8; 4] array (little-endian, matches RGBA packing), flattens the arrays into a single sequence
            .collect() // gathers into Vec<u8>
    }
}
