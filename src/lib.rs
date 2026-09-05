//! mouse-coords: pega a coordenada exata do mouse.
//!
//! ```rust
//! let pos = mouse_coords::get_position().unwrap();
//! println!("x={} y={}", pos.x, pos.y);
//! ```

mod error;
mod platform;

pub use error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Retorna a posição global do cursor em pixels, origem (0,0) no canto
/// superior-esquerdo (multi-monitor: soma os offsets, ex: 3840x1080 = dois 1920 lado a lado).
pub fn get_position() -> Result<Point, Error> {
    platform::get_position()
}

/// Alias orientado a objeto (compatível com a API da crate `mouse_position`).
pub struct Mouse;

impl Mouse {
    pub fn get_mouse_position() -> Result<Point, Error> {
        get_position()
    }
}
