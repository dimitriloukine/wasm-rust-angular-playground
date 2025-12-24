// Procedural pattern generation - infinite textures computed on-the-fly
// Docs: https://en.wikipedia.org/wiki/Procedural_generation

/// RGBA color as [R, G, B, A] where each component is 0-255
type Color = [u8; 4];

/// Generate a checkerboard pattern at the given position
/// Returns red or white based on alternating squares
pub fn checkerboard(x: u32, y: u32, square_size: u32) -> Color {
    let square_x = x / square_size;
    let square_y = y / square_size;

    if (square_x + square_y) % 2 == 0 {
        [255, 0, 0, 255] // Red
    } else {
        [255, 255, 255, 255] // White
    }
}
