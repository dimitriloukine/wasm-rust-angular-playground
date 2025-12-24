use crate::math::Vec2;
use wasm_bindgen::prelude::*;

// Struct to hold our software renderer state - Docs: https://doc.rust-lang.org/book/ch05-00-structs.html
#[wasm_bindgen]
pub struct SoftwareRenderer {
    pixels: Vec<u8>, // Our pixel buffer (RGBA format)
    width: u32,
    height: u32,
    square_size: u32,
    offset: Vec2,   // Scroll offset (x, y)
    velocity: Vec2, // Current velocity (x, y) for smooth easing
}

// Implementation block - methods for our struct - Docs: https://doc.rust-lang.org/book/ch05-03-method-syntax.html
#[wasm_bindgen]
impl SoftwareRenderer {
    // Constructor - called from JS with SoftwareRenderer.new() - Docs: https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-rust-exports.html
    pub fn new(width: u32, height: u32, square_size: u32) -> Self {
        let pixels = Vec::with_capacity((width * height * 4) as usize);
        Self {
            pixels,
            width,
            height,
            square_size,
            offset: Vec2::zero(),
            velocity: Vec2::zero(),
        }
    }

    // Update animation based on time elapsed and pressed keys
    pub fn update(&mut self, delta_time_ms: u32, keys: Vec<String>) {
        // Process keys to determine target velocity
        let mut target_velocity = Vec2::zero();
        let speed = 960.0; // pixels per second

        for key in keys {
            match key.as_str() {
                "ArrowUp" | "w" => target_velocity.y -= speed,
                "ArrowDown" | "s" => target_velocity.y += speed,
                "ArrowLeft" | "a" => target_velocity.x -= speed,
                "ArrowRight" | "d" => target_velocity.x += speed,
                _ => {}
            }
        }

        let delta_seconds: f32 = delta_time_ms as f32 / 1000.0;

        // Smoothly interpolate current velocity towards target velocity
        let acceleration: f32 = 800.0; // pixels per second squared (how fast we accelerate)
        let friction: f32 = 0.97; // damping factor (closer to 0 = more friction)

        if target_velocity.x != 0.0 || target_velocity.y != 0.0 {
            // Accelerate towards target velocity when keys are pressed
            self.velocity.x +=
                (target_velocity.x - self.velocity.x) * acceleration * delta_seconds / 100.0;
            self.velocity.y +=
                (target_velocity.y - self.velocity.y) * acceleration * delta_seconds / 100.0;
        } else {
            // Apply friction when no keys are pressed (ease out)
            self.velocity.x *= friction.powf(delta_seconds * 60.0); // Normalize for framerate
            self.velocity.y *= friction.powf(delta_seconds * 60.0);

            // Stop completely when velocity is very small
            if self.velocity.x.abs() < 0.1 {
                self.velocity.x = 0.0;
            }
            if self.velocity.y.abs() < 0.1 {
                self.velocity.y = 0.0;
            }
        }

        // Update offset based on current velocity
        self.offset.x += self.velocity.x * delta_seconds;
        self.offset.y += self.velocity.y * delta_seconds;
    }

    // Render a frame - updates the pixel buffer
    pub fn render_frame(&mut self) {
        // &mut self = mutable reference to this instance - Docs: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
        self.pixels.clear(); // Clear previous frame

        for y in 0..self.height {
            for x in 0..self.width {
                // Apply scrolling offset with wrapping (modulo prevents underflow)
                let scroll_x = ((x as f32 - self.offset.x).rem_euclid(self.width as f32)) as u32;
                let scroll_y = ((y as f32 - self.offset.y).rem_euclid(self.height as f32)) as u32;

                let square_x = scroll_x / self.square_size;
                let square_y = scroll_y / self.square_size;
                let is_red = (square_x + square_y) % 2 == 0;

                if is_red {
                    self.pixels.push(255); // R
                    self.pixels.push(0); // G
                    self.pixels.push(0); // B
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

    // Get pixel data as a JS-accessible slice
    pub fn get_pixels(&self) -> Vec<u8> {
        self.pixels.clone()
    }

    // Getters for dimensions
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
