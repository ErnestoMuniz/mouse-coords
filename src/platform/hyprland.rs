//! Cursor position on Hyprland (Wayland) via the Hyprland IPC socket.
//!
//! Hyprland exposes the global pointer through its control socket. The
//! protocol is one request per connection: write the command, read the
//! reply until EOF. We use `cursorpos`, which answers `"<x>, <y>"`.
//!
//! Socket location (same lookup as `hyprctl`):
//! 1. `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
//! 2. `/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
//!
//! If `HYPRLAND_INSTANCE_SIGNATURE` is unset (e.g. launched from a service
//! that did not import it), the first `.socket.sock` found under either
//! base directory is used.
//!
//! This is a permission-free native API — unlike `XQueryPointer` on
//! XWayland, which is frozen while the cursor is not over an X11 window.

use crate::{Error, Point};
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const SOCKET_TIMEOUT: Duration = Duration::from_secs(2);
const CURSOR_POS_REQUEST: &[u8] = b"cursorpos";

/// Parses the `cursorpos` reply, e.g. `"1920, 540"`.
fn parse_cursor_pos(reply: &str) -> Option<(i32, i32)> {
    let (x, y) = reply.trim().split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// Base directories that may contain `hypr/<signature>/.socket.sock`.
fn socket_bases() -> Vec<PathBuf> {
    let mut bases = Vec::new();
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime.trim().is_empty() {
            bases.push(PathBuf::from(runtime).join("hypr"));
        }
    }
    bases.push(PathBuf::from("/tmp/hypr"));
    bases
}

/// Finds the IPC socket, preferring the instance named by
/// `HYPRLAND_INSTANCE_SIGNATURE`.
fn socket_path() -> Result<PathBuf, Error> {
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .ok()
        .filter(|s| !s.trim().is_empty());

    if let Some(sig) = &signature {
        for base in socket_bases() {
            let candidate = base.join(sig).join(".socket.sock");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    // Fallback: no/unknown signature, scan for any Hyprland control socket.
    for base in socket_bases() {
        let Ok(entries) = std::fs::read_dir(&base) else {
            continue;
        };
        for entry in entries.flatten() {
            let candidate = entry.path().join(".socket.sock");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    let detail = match signature {
        Some(sig) => format!("HYPRLAND_INSTANCE_SIGNATURE={sig:?}"),
        None => "HYPRLAND_INSTANCE_SIGNATURE is not set".to_string(),
    };
    Err(Error::Connection(format!(
        "Hyprland IPC socket not found ({detail}); is Hyprland running and is \
         $XDG_RUNTIME_DIR available?"
    )))
}

fn query(path: &Path) -> Result<Point, Error> {
    let mut stream = UnixStream::connect(path)
        .map_err(|e| Error::Connection(format!("Hyprland IPC connect {}: {e}", path.display())))?;
    stream
        .set_read_timeout(Some(SOCKET_TIMEOUT))
        .map_err(|e| Error::Query(format!("set_read_timeout: {e}")))?;
    stream
        .set_write_timeout(Some(SOCKET_TIMEOUT))
        .map_err(|e| Error::Query(format!("set_write_timeout: {e}")))?;

    stream
        .write_all(CURSOR_POS_REQUEST)
        .map_err(|e| Error::Query(format!("Hyprland IPC write: {e}")))?;
    // Signal end of request so the compositor replies and closes.
    let _ = stream.shutdown(Shutdown::Write);

    let mut buf = [0u8; 128];
    let mut filled = 0;
    loop {
        match stream.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => {
                filled += n;
                if filled == buf.len() {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            // A read timeout with data already buffered is still usable.
            Err(e) if filled > 0 => {
                let _ = e;
                break;
            }
            Err(e) => return Err(Error::Query(format!("Hyprland IPC read: {e}"))),
        }
    }

    let reply = String::from_utf8_lossy(&buf[..filled]);
    match parse_cursor_pos(&reply) {
        Some((x, y)) => Ok(Point { x, y }),
        None => Err(Error::Query(format!(
            "Hyprland cursorpos returned an unparseable reply: {reply:?}"
        ))),
    }
}

/// Blocking query (no async runtime needed; the Hyprland protocol is a
/// single request/response over a Unix socket).
pub fn get_position_blocking() -> Result<Point, Error> {
    let path = socket_path()?;
    query(&path)
}

#[cfg(test)]
mod tests {
    use super::parse_cursor_pos;

    #[test]
    fn parses_plain() {
        assert_eq!(parse_cursor_pos("1920, 540"), Some((1920, 540)));
    }

    #[test]
    fn parses_spacing_and_newline() {
        assert_eq!(parse_cursor_pos(" 12,34\n"), Some((12, 34)));
        assert_eq!(parse_cursor_pos("0, 0"), Some((0, 0)));
    }

    #[test]
    fn parses_negative_multi_monitor() {
        assert_eq!(parse_cursor_pos("-1920, -1080"), Some((-1920, -1080)));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_cursor_pos("unknown request"), None);
        assert_eq!(parse_cursor_pos("1.5, 2"), None);
        assert_eq!(parse_cursor_pos(""), None);
    }
}
