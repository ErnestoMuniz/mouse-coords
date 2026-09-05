use crate::{Error, Point};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

#[cfg(target_os = "linux")]
use super::kwin;

/// Plain X11 via `XQueryPointer` on the root window.
///
/// NOTE: on Wayland + XWayland this value is frozen (it only updates
/// while the cursor is over an X11 window). That is why, on a Wayland
/// + KDE session, `get_position()` prefers the native KWin backend below.
pub fn get_position_x11() -> Result<Point, Error> {
    let (conn, screen_num) =
        x11rb::connect(None).map_err(|e| Error::Connection(format!("x11rb::connect: {e}")))?;

    let root = conn.setup().roots[screen_num].root;

    let reply = conn
        .query_pointer(root)
        .map_err(|e| Error::Query(format!("query_pointer: {e}")))?
        .reply()
        .map_err(|e| Error::Query(format!("reply: {e}")))?;

    Ok(Point {
        x: reply.root_x as i32,
        y: reply.root_y as i32,
    })
}

fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|v| v.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
        || std::env::var("WAYLAND_DISPLAY").is_ok()
}

fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .map(|v| v.to_ascii_lowercase().contains("kde"))
        .unwrap_or(false)
}

/// Linux: Wayland+KDE -> KWin scripting (`workspace.cursorPos`, accurate);
/// everything else -> X11 (`XQueryPointer`).
pub fn get_position() -> Result<Point, Error> {
    if is_wayland() && is_kde() {
        match kwin::get_position_blocking() {
            Ok(p) => return Ok(p),
            Err(e) => {
                eprintln!("mouse-coords: kwin failed ({e}), trying X11 as fallback");
            }
        }
    }
    get_position_x11()
}
