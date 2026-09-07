//! Cursor position on GNOME Shell (Wayland).
//!
//! Probe order (first success wins):
//! 1. Companion Shell extension exposing `GetPointerPosition() -> (x, y)`.
//!    Tried on both bus names (same method shape):
//!    - `org.wdotool.GnomeShellBridge` (existing wdotool extension, GNOME 45-48)
//!    - `org.mousecoords.Bridge` (extension shipped in
//!      `packaging/gnome-extension/`, declares GNOME 45-50)
//!
//!    Extensions run inside the compositor, so this needs no unsafe mode.
//! 2. `org.gnome.Shell.Eval("global.get_pointer()")`, which works when
//!    unsafe mode is enabled (`global.context.unsafe_mode = true`, e.g. via
//!    Looking Glass `Alt+F2 lg`, the "Unsafe mode menu" extension, or
//!    `gnome-shell --unsafe-mode`). Since GNOME 41 `Eval` returns
//!    `(false, "")` out of the box.
//!
//! NOTE: stock GNOME Wayland has **no permission-free public API** for the
//! global pointer (layer-shell / virtual-pointer were rejected by GNOME,
//! legacy AT-SPI device-event sniffing does not work on Wayland, libei /
//! RemoteDesktop portal is input-only, and `XQueryPointer` on XWayland is
//! frozen, usually at the screen center). When nothing above is available we
//! return a descriptive error instead of a frozen coordinate.

use crate::{Error, Point};
use std::time::Duration;

#[zbus::proxy(
    interface = "org.gnome.Shell",
    default_service = "org.gnome.Shell",
    default_path = "/org/gnome/Shell"
)]
trait GnomeShell {
    #[zbus(name = "Eval")]
    fn eval(&self, script: &str) -> zbus::Result<(bool, String)>;
}

/// Shape of the companion-extension bridge (wdotool-compatible).
#[zbus::proxy(
    interface = "org.wdotool.GnomeShellBridge",
    default_service = "org.wdotool.GnomeShellBridge",
    default_path = "/org/wdotool/GnomeShellBridge"
)]
trait WdotoolBridge {
    #[zbus(name = "GetPointerPosition")]
    fn get_pointer_position(&self) -> zbus::Result<(i32, i32)>;
}

/// Shape of the bridge shipped in `packaging/gnome-extension/`
/// (same method, own bus name so it works even if wdotool is absent).
#[zbus::proxy(
    interface = "org.mousecoords.Bridge",
    default_service = "org.mousecoords.Bridge",
    default_path = "/org/mousecoords/Bridge"
)]
trait MouseCoordsBridge {
    #[zbus(name = "GetPointerPosition")]
    fn get_pointer_position(&self) -> zbus::Result<(i32, i32)>;
}

fn dbus_err(e: zbus::Error) -> Error {
    Error::Query(format!("gnome dbus: {e}"))
}

/// Parses the string returned by `global.get_pointer()`.
///
/// Expected forms: `[123, 456, 0]`, `[123.0, 456.0, 0]`,
/// possibly wrapped in extra quotes.
fn parse_pointer_result(s: &str) -> Option<(i32, i32)> {
    let mut t = s.trim();
    // Eval result may arrive wrapped in single/double quotes.
    if (t.starts_with('\'') && t.ends_with('\'')) || (t.starts_with('"') && t.ends_with('"')) {
        t = t[1..t.len().saturating_sub(1)].trim();
    }
    // Keep only the bracketed list if there is surrounding text.
    if let (Some(a), Some(b)) = (t.find('['), t.rfind(']')) {
        if a < b {
            t = &t[a + 1..b];
        }
    }
    let mut parts = t.split(',');
    let x: f64 = parts.next()?.trim().parse().ok()?;
    let y: f64 = parts.next()?.trim().parse().ok()?;
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    Some((x.round() as i32, y.round() as i32))
}

const NO_BACKEND_MSG: &str = "GNOME Wayland: no pointer backend available (tried companion \
     extension bridges org.wdotool.GnomeShellBridge and org.mousecoords.Bridge, then Shell Eval, \
     which returned false). Stock GNOME has no permission-free global-pointer API. Pick one \
     (no X11 session switch needed): (1) install the companion extension from \
     packaging/gnome-extension/ (copy to ~/.local/share/gnome-shell/extensions/, log out/in, \
     `gnome-extensions enable mousecoords@mouse-coords.github.io`); or the wdotool extension \
     (GNOME 45-48); (2) enable unsafe mode for this session via Looking Glass (Alt+F2, `lg`, \
     `global.context.unsafe_mode = true`) and retry — no install needed; (3) use KDE Plasma \
     Wayland (KWin backend) or a GNOME on Xorg session (plain X11). XWayland XQueryPointer is \
     frozen (often the screen center, e.g. x=640 y=400 on 1280x800) and is not used.";

pub async fn get_position_async() -> Result<Point, Error> {
    let conn = zbus::Connection::session().await.map_err(dbus_err)?;

    // 1a. wdotool companion extension (GNOME 45-48), if installed+enabled.
    // Proxies connect lazily, so the real probe is the method call itself;
    // ServiceUnknown (absent) fails fast and falls through. The timeout
    // guards against a name owner that never replies.
    if let Ok(bridge) = WdotoolBridgeProxy::new(&conn).await {
        if let Ok(Ok((x, y))) =
            tokio::time::timeout(Duration::from_secs(2), bridge.get_pointer_position()).await
        {
            return Ok(Point { x, y });
        }
    }

    // 1b. mouse-coords companion extension (declares GNOME 45-50).
    if let Ok(bridge) = MouseCoordsBridgeProxy::new(&conn).await {
        if let Ok(Ok((x, y))) =
            tokio::time::timeout(Duration::from_secs(2), bridge.get_pointer_position()).await
        {
            return Ok(Point { x, y });
        }
    }

    // 2. Shell Eval (needs unsafe mode since GNOME 41).
    let shell = GnomeShellProxy::new(&conn).await.map_err(dbus_err)?;
    let (ok, result) = shell.eval("global.get_pointer()").await.map_err(dbus_err)?;
    if !ok {
        return Err(Error::Query(NO_BACKEND_MSG.into()));
    }
    match parse_pointer_result(&result) {
        Some((x, y)) => Ok(Point { x, y }),
        None => Err(Error::Query(format!(
            "GNOME Shell Eval returned an unparseable pointer: {result:?}"
        ))),
    }
}

/// Sync wrapper: runs the runtime on a dedicated thread so it works
/// both outside and inside an existing tokio runtime.
pub fn get_position_blocking() -> Result<Point, Error> {
    std::thread::spawn(move || {
        match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt.block_on(get_position_async()),
            Err(e) => Err(Error::Query(format!("tokio rt: {e}"))),
        }
    })
    .join()
    .map_err(|_| Error::Query("gnome thread panicked".into()))?
}

#[cfg(test)]
mod tests {
    use super::parse_pointer_result;

    #[test]
    fn parses_int_list() {
        assert_eq!(parse_pointer_result("[640, 400, 0]"), Some((640, 400)));
    }

    #[test]
    fn parses_float_list() {
        assert_eq!(parse_pointer_result("[640.0, 400.0, 0]"), Some((640, 400)));
    }

    #[test]
    fn parses_quoted() {
        assert_eq!(parse_pointer_result("'[12, 34, 0]'"), Some((12, 34)));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_pointer_result(""), None);
        assert_eq!(parse_pointer_result("false"), None);
    }
}
