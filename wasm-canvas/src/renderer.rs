use crate::math::Vec2;
use crate::pattern;
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

// Number of entries in sine/cosine lookup tables (power of 2 for fast modulo via bitwise AND)
const ANGLE_STEPS: usize = 1024;
const ANGLE_MASK: usize = ANGLE_STEPS - 1; // For wrapping: angle & ANGLE_MASK

// Perspective information for each scanline (Mode 7 / floor rendering)
struct ScanlineInfo {
    depth: f32,        // World-space distance to this row
    texture_step: f32, // Texture coordinate increment per screen pixel
}

// Struct to hold our software renderer state - Docs: https://doc.rust-lang.org/book/ch05-00-structs.html
#[wasm_bindgen]
pub struct SoftwareRenderer {
    pixels: Vec<u32>, // Our pixel buffer (packed RGBA as 32-bit integers)
    tile: Vec<u32>,   // Pre-computed repeating pattern tile
    tile_size: u32,   // Size of the tile (power of 2 for fast wrapping)
    tile_mask: u32,   // Bitmask for wrapping (tile_size - 1)
    width: u32,
    height: u32,
    horizon_y: u32,                  // Y coordinate of horizon line
    floor_start_y: u32, // Y coordinate where floor rendering starts (after depth clamp)
    scanline_lut: Vec<ScanlineInfo>, // Pre-computed perspective data per scanline
    sin_table: Vec<f32>, // Pre-computed sine values [0..ANGLE_STEPS]
    cos_table: Vec<f32>, // Pre-computed cosine values [0..ANGLE_STEPS]
    angle: usize,       // Current rotation angle (index into sin/cos tables)
    offset: Vec2,       // Camera position in world space (x, z)
    velocity: Vec2,     // Current velocity (x, z) for smooth easing
    render_start_time: f64, // Timestamp when update() was called
    last_render_time_ms: f64, // Actual time taken to render last frame (profiling)
}

// Implementation block - methods for our struct - Docs: https://doc.rust-lang.org/book/ch05-03-method-syntax.html
#[wasm_bindgen]
impl SoftwareRenderer {
    // Constructor - called from JS with SoftwareRenderer.new() - Docs: https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-rust-exports.html
    pub fn new(width: u32, height: u32, square_size: u32) -> Self {
        // Allocate pixel buffer once with exact size (never resize during rendering)
        let pixel_count = (width * height) as usize;
        let pixels = vec![0u32; pixel_count];

        // Generate pre-computed tile using pattern module
        // This separates pattern logic from rendering infrastructure
        let (tile, tile_size) = pattern::generate_checkerboard_tile(square_size);

        // Tile size is guaranteed to be a power of 2, so we can use bitwise AND for wrapping
        // This is much faster than rem_euclid (1 bitwise op vs division)
        let tile_mask = tile_size - 1;

        // Perspective rendering setup (Mode 7 style)
        let horizon_y = height / 3; // Horizon in upper third (raises camera viewpoint)
        let camera_height = 100.0; // Height of camera above ground plane
        let focal_length = 200.0; // Controls FOV (higher = narrower)
        let max_depth = 700.0; // Clamp far field to prevent aliasing/moiré

        // Pre-compute perspective data for each scanline from horizon to bottom
        let scanline_count = (height - horizon_y) as usize;
        let mut scanline_lut = Vec::with_capacity(scanline_count);
        let mut floor_start_y = horizon_y; // Track where floor actually starts

        for row in 0..scanline_count {
            // Distance from camera increases as we go down the screen
            // row 0 = horizon (far away), row (height-horizon_y) = bottom (close)
            let screen_y = (row as f32) + 0.5; // Add 0.5 for pixel center

            // Perspective formula: depth = (camera_height * focal_length) / screen_y
            // Small screen_y (near horizon) → large depth (far away)
            // Large screen_y (bottom) → small depth (close to camera)
            let depth = (camera_height * focal_length) / screen_y;

            // Skip scanlines beyond max_depth (too far = aliasing artifacts)
            if depth > max_depth {
                floor_start_y = horizon_y + row as u32 + 1;
                continue;
            }

            // Texture step: how much world space one screen pixel covers
            // Larger depth = more world space per pixel (tiles look bigger/closer)
            let texture_step = depth / focal_length;

            scanline_lut.push(ScanlineInfo {
                depth,
                texture_step,
            });
        }

        // Pre-compute sine/cosine lookup tables for rotation
        // This avoids expensive trig calls during rendering (classic game optimization)
        let mut sin_table = Vec::with_capacity(ANGLE_STEPS);
        let mut cos_table = Vec::with_capacity(ANGLE_STEPS);
        for i in 0..ANGLE_STEPS {
            let angle_radians = (i as f32) * 2.0 * PI / (ANGLE_STEPS as f32);
            sin_table.push(angle_radians.sin());
            cos_table.push(angle_radians.cos());
        }

        Self {
            pixels,
            tile,
            tile_size,
            tile_mask,
            width,
            height,
            horizon_y,
            floor_start_y,
            scanline_lut,
            sin_table,
            cos_table,
            angle: 0,
            offset: Vec2::zero(),
            velocity: Vec2::zero(),
            render_start_time: 0.0,
            last_render_time_ms: 0.0,
        }
    }

    // Constructor with external texture data from PNG/image file
    pub fn new_with_texture(
        width: u32,
        height: u32,
        texture_size: u32,
        texture_data: Vec<u8>,
    ) -> Self {
        // Allocate pixel buffer once with exact size (never resize during rendering)
        let pixel_count = (width * height) as usize;
        let pixels = vec![0u32; pixel_count];

        // Convert RGBA u8 array from JavaScript to packed u32 format
        // JavaScript ImageData is RGBA, we pack to u32 little-endian (ABGR in memory)
        let tile_pixel_count = (texture_size * texture_size) as usize;
        let mut tile = Vec::with_capacity(tile_pixel_count);

        for chunk in texture_data.chunks_exact(4) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let a = chunk[3];
            // Pack as little-endian u32: ABGR in memory order
            let packed = ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32);
            tile.push(packed);
        }

        // Texture size must be power of 2 for fast bitwise wrapping
        let tile_mask = texture_size - 1;

        // Perspective rendering setup (Mode 7 style)
        let horizon_y = height / 3;
        let camera_height = 100.0;
        let focal_length = 200.0;
        let max_depth = 700.0;

        // Pre-compute perspective data for each scanline
        let scanline_count = (height - horizon_y) as usize;
        let mut scanline_lut = Vec::with_capacity(scanline_count);
        let mut floor_start_y = horizon_y;

        for row in 0..scanline_count {
            let screen_y = (row as f32) + 0.5;
            let depth = (camera_height * focal_length) / screen_y;

            if depth > max_depth {
                floor_start_y = horizon_y + row as u32 + 1;
                continue;
            }

            let texture_step = depth / focal_length;
            scanline_lut.push(ScanlineInfo {
                depth,
                texture_step,
            });
        }

        // Pre-compute sine/cosine lookup tables
        let mut sin_table = Vec::with_capacity(ANGLE_STEPS);
        let mut cos_table = Vec::with_capacity(ANGLE_STEPS);
        for i in 0..ANGLE_STEPS {
            let angle_radians = (i as f32) * 2.0 * PI / (ANGLE_STEPS as f32);
            sin_table.push(angle_radians.sin());
            cos_table.push(angle_radians.cos());
        }

        Self {
            pixels,
            tile,
            tile_size: texture_size,
            tile_mask,
            width,
            height,
            horizon_y,
            floor_start_y,
            scanline_lut,
            sin_table,
            cos_table,
            angle: 0,
            offset: Vec2::zero(),
            velocity: Vec2::zero(),
            render_start_time: 0.0,
            last_render_time_ms: 0.0,
        }
    }

    // Update animation based on time elapsed and pressed keys
    pub fn update(&mut self, delta_time_ms: u32, keys: Vec<String>) {
        // Start profiling timer
        self.render_start_time = web_sys::window().unwrap().performance().unwrap().now();

        // Process keys to determine target velocity
        let mut target_velocity = Vec2::zero();
        let speed = 960.0; // pixels per second
        let mut rotation_change = 0i32; // Change in angle this frame

        for key in keys {
            match key.as_str() {
                "ArrowUp" | "z" => target_velocity.y += speed,
                "ArrowDown" | "s" => target_velocity.y -= speed,
                "ArrowLeft" | "q" => rotation_change += 2, // Rotate counter-clockwise
                "ArrowRight" | "d" => rotation_change -= 2, // Rotate clockwise
                _ => {}
            }
        }

        // Update rotation angle using lookup table indices (no trig needed!)
        self.angle = ((self.angle as i32 + rotation_change) & ANGLE_MASK as i32) as usize;

        let delta_seconds: f32 = delta_time_ms as f32 / 1000.0;

        // Transform movement based on current rotation (forward = direction we're facing)
        // Use lookup tables instead of sin/cos computation
        let cos_a = self.cos_table[self.angle];
        let sin_a = self.sin_table[self.angle];

        // Rotate velocity vector by current angle
        let rotated_velocity_x = target_velocity.x * cos_a - target_velocity.y * sin_a;
        let rotated_velocity_y = target_velocity.x * sin_a + target_velocity.y * cos_a;
        let rotated_target = Vec2 {
            x: rotated_velocity_x,
            y: rotated_velocity_y,
        };

        // Smoothly interpolate current velocity towards target velocity
        let acceleration: f32 = 800.0; // pixels per second squared (how fast we accelerate)
        let friction: f32 = 0.90; // damping factor per frame (closer to 0 = more friction)

        if rotated_target.x != 0.0 || rotated_target.y != 0.0 {
            // Accelerate towards target velocity when keys are pressed
            self.velocity.x +=
                (rotated_target.x - self.velocity.x) * acceleration * delta_seconds / 100.0;
            self.velocity.y +=
                (rotated_target.y - self.velocity.y) * acceleration * delta_seconds / 100.0;
        } else if self.velocity.x != 0.0 || self.velocity.y != 0.0 {
            // Apply friction when no keys are pressed (ease out)
            // Simple multiply is much faster than powf() and more consistent
            self.velocity.x *= friction;
            self.velocity.y *= friction;

            // Stop completely when velocity is very small
            if self.velocity.x.abs() < 1.0 {
                self.velocity.x = 0.0;
            }
            if self.velocity.y.abs() < 1.0 {
                self.velocity.y = 0.0;
            }
        }

        // Update offset based on current velocity
        self.offset.x += self.velocity.x * delta_seconds;
        self.offset.y += self.velocity.y * delta_seconds;

        // Wrap offset to prevent unbounded growth and maintain performance
        // Large float values cause slower arithmetic operations
        self.offset.x = self.offset.x.rem_euclid(self.tile_size as f32);
        self.offset.y = self.offset.y.rem_euclid(self.tile_size as f32);
    }

    // Render a frame - updates the pixel buffer
    pub fn render_frame(&mut self) {
        // &mut self = mutable reference to this instance - Docs: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html

        // OPTIMIZATION: Perspective floor rendering (Mode 7 / SNES style)
        // Uses pre-computed scanline lookup table for depth and texture stepping
        // Each row samples texture at different scale based on distance
        const SKY_COLOR: u32 = 0xFF_EB_CE_87; // Light blue sky (ABGR in little-endian)

        // Camera position and orientation in world space
        let camera_x = self.offset.x;
        let camera_z = self.offset.y;

        // Get rotation from lookup tables (no trig computation needed!)
        let cos_a = self.cos_table[self.angle];
        let sin_a = self.sin_table[self.angle];

        // Render sky (everything above floor_start_y, including clamped far field)
        // Use fill() for better performance than individual pixel writes
        let sky_pixel_count = (self.floor_start_y * self.width) as usize;
        self.pixels[..sky_pixel_count].fill(SKY_COLOR);

        // Render perspective floor (only scanlines within depth range)
        for y in self.floor_start_y..self.height {
            let scanline_idx = (y - self.floor_start_y) as usize;
            let info = &self.scanline_lut[scanline_idx];

            // OPTIMIZATION: Rotate the step vector once per scanline, not per pixel
            // This reduces per-pixel work from 4 multiplies to 2 additions

            // In camera space: starting position (left edge) and step per pixel
            let cam_x_start = -(self.width as f32 / 2.0) * info.texture_step;
            let cam_z = info.depth;
            let cam_x_step = info.texture_step;

            // Rotate the starting position by camera angle
            let rotated_start_x = cam_x_start * cos_a - cam_z * sin_a;
            let rotated_start_z = cam_x_start * sin_a + cam_z * cos_a;

            // Rotate the step vector by camera angle (how much world pos changes per screen pixel)
            let rotated_step_x = cam_x_step * cos_a;
            let rotated_step_z = cam_x_step * sin_a;

            // Translate to camera position
            let mut world_x = camera_x + rotated_start_x;
            let mut world_z = camera_z + rotated_start_z;

            let screen_row_start = (y * self.width) as usize;

            // Sample texture for each pixel in this scanline
            // Incrementally add the rotated step (just 2 additions per pixel!)
            for x in 0..self.width {
                // Fast wrapping using bitwise AND (tile_size is power of 2)
                // This replaces expensive rem_euclid with single bitwise operation
                let tile_x = (world_x as i32 & self.tile_mask as i32) as usize;
                let tile_z = (world_z as i32 & self.tile_mask as i32) as usize;

                // Lookup pixel from tile
                let tile_idx = tile_z * self.tile_size as usize + tile_x;
                self.pixels[screen_row_start + x as usize] = self.tile[tile_idx];

                // Step to next pixel (classic DDA / incremental technique)
                world_x += rotated_step_x;
                world_z += rotated_step_z;
            }
        }

        // End profiling timer and store elapsed time
        let end_time = web_sys::window().unwrap().performance().unwrap().now();
        self.last_render_time_ms = end_time - self.render_start_time;
    }

    // Get pixel data as a JS-accessible slice
    // Converts our u32 buffer to u8 bytes that WebGL expects
    pub fn get_pixels(&self) -> Vec<u8> {
        // Reinterpret u32 buffer as u8 bytes
        // Each u32 becomes 4 consecutive u8 values (RGBA)

        // Safe because we're just changing how we view the same memory
        // unsafe {
        //     std::slice::from_raw_parts(self.pixels.as_ptr() as *const u8, self.pixels.len() * 4)
        //         .to_vec()
        // }

        self.pixels
            .iter()
            .flat_map(|&pixel| pixel.to_le_bytes())
            .collect()
    }

    // Getters for dimensions
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    // Get last frame time in milliseconds
    pub fn frame_time(&self) -> f64 {
        self.last_render_time_ms
    }
}
