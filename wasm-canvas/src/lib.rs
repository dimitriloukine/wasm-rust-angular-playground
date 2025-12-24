// Main library entry point - declares modules and re-exports public API
// Docs: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html

mod math; // Vec2 struct for 2D vectors
mod renderer; // SoftwareRenderer implementation

// Re-export types that need to be accessible from JavaScript
pub use renderer::SoftwareRenderer;
