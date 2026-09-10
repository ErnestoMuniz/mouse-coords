use crate::{Error, Point};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

#[cfg(target_os = "linux")]
use super::gnome;
#[cfg(target_os = "linux")]
use super::hyprland;
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
    // XDG_SESSION_TYPE is authoritative when present. WAYLAND_DISPLAY is a
    // fallback, but must be non-empty (an empty/unset value means X11).
    if let Ok(v) = std::env::var("XDG_SESSION_TYPE") {
        if v.eq_ignore_ascii_case("wayland") {
            return true;
        }
        if v.eq_ignore_ascii_case("x11") {
            return false;
        }
    }
    std::env::var("WAYLAND_DISPLAY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .map(|v| {
            let v = v.to_ascii_lowercase();
            v.contains("kde") || v.contains("plasma")
        })
        .unwrap_or(false)
}

fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .map(|v| v.to_ascii_lowercase().contains("gnome"))
        .unwrap_or(false)
}

fn is_hyprland() -> bool {
    // The instance signature is only set inside a Hyprland session, so it is
    // the most reliable signal even if XDG_CURRENT_DESKTOP was overridden.
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        return true;
    }
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .map(|v| v.to_ascii_lowercase().contains("hyprland"))
        .unwrap_or(false)
}

/// Linux routing:
/// - Wayland + KDE -> KWin scripting (`workspace.cursorPos`, accurate).
/// - Wayland + GNOME -> GNOME Shell `Eval(global.get_pointer())`.
/// - Wayland + Hyprland -> Hyprland IPC socket (`cursorpos`).
/// - Wayland + anything else -> error (do NOT fall back to X11:
///   `XQueryPointer` on XWayland is frozen, often the screen center,
///   e.g. x=640 y=400 on 1280x800).
/// - Plain X11 (no Wayland) -> X11 (`XQueryPointer`).
pub fn get_position() -> Result<Point, Error> {
    if is_wayland() {
        if is_kde() {
            // Do not fall back to X11 here: on Wayland it would return a
            // frozen coordinate instead of an error.
            return kwin::get_position_blocking();
        }
        if is_gnome() {
            return gnome::get_position_blocking();
        }
        if is_hyprland() {
            return hyprland::get_position_blocking();
        }
        return Err(Error::Unsupported(
            "Wayland session without a supported compositor backend (only KDE/KWin, GNOME \
             Shell Eval and Hyprland IPC are attempted). XQueryPointer on XWayland would \
             return a frozen position (often the screen center), so it is not used as a \
             fallback. Use KDE Plasma Wayland, GNOME Wayland, Hyprland, a GNOME on Xorg \
             session, or a compositor-specific API (e.g. Sway IPC)."
                .into(),
        ));
    }
    get_position_x11()
}
