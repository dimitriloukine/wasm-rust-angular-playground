// Procedural pattern generation - infinite textures computed on-the-fly
// Docs: https://en.wikipedia.org/wiki/Procedural_generation

/// Generate a pre-computed checkerboard tile (2×2 squares pattern)
/// This creates a repeating tile that can be used for fast rendering
/// Returns a tuple of (tile_pixels, tile_size) where each pixel is packed RGBA as u32
/// NOTE: tile_size will be rounded up to nearest power of 2 for fast wrapping
pub fn generate_checkerboard_tile(square_size: u32) -> (Vec<u32>, u32) {
    const RED_PIXEL: u32 = 0xFF_00_00_FF; // RGBA red
    const WHITE_PIXEL: u32 = 0xFF_FF_FF_FF; // RGBA white

    // Tile covers 2×2 squares (one full repeating pattern)
    // Round up to nearest power of 2 for fast bitwise wrapping
    let desired_size = square_size * 2;
    let tile_size = desired_size.next_power_of_two();
    let tile_pixels = (tile_size * tile_size) as usize;
    let mut tile = vec![0u32; tile_pixels];

    // Fill the tile with checkerboard pattern
    for y in 0..tile_size {
        for x in 0..tile_size {
            let square_x = x / square_size;
            let square_y = y / square_size;
            let is_red = (square_x + square_y) % 2 == 0;
            let idx = (y * tile_size + x) as usize;
            tile[idx] = if is_red { RED_PIXEL } else { WHITE_PIXEL };
        }
    }

    (tile, tile_size)
}
