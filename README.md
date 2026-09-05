# mouse-coords

Get the exact mouse coordinates in Rust.

```rust
let pos = mouse_coords::get_position().unwrap();
println!("x={} y={}", pos.x, pos.y);
```

Or run the example:

```bash
cargo run --example pos
cargo run --example pos -- --loop  # poll every 200ms, move the mouse to see it change
```

## How it works (Linux)

- **KDE Plasma Wayland** (e.g. Fedora KDE): KWin scripting over D-Bus
  (`workspace.cursorPos` + `callDBus` callback, same technique as
  `kdotool`/`wdotool`). It is the default backend when
  `XDG_SESSION_TYPE=wayland` + KDE desktop. Accurate, follows the mouse.
- **Plain X11**: `XQueryPointer` on the root window via `x11rb`.
- **Wayland + XWayland without KWin**: `XQueryPointer` returns a
  **frozen** position (it only updates over X11 windows) — that is why KWin
  is preferred on KDE. Do not use X11 as a reference on Wayland.

## Other platforms

- **Windows**: `GetCursorPos`.
- **macOS**: `CGEventCreate` + `CGEventGetLocation`.
