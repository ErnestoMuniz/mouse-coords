//! mouse-coords: get the exact mouse coordinates.
//!
//! ```rust,no_run
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

/// Returns the global cursor position in pixels, origin (0,0) at the
/// top-left corner (multi-monitor: offsets are summed, e.g. 3840x1080 = two 1920 side by side).
pub fn get_position() -> Result<Point, Error> {
    platform::get_position()
}

/// Object-oriented alias (compatible with the `mouse_position` crate API).
pub struct Mouse;

impl Mouse {
    pub fn get_mouse_position() -> Result<Point, Error> {
        get_position()
    }
}
