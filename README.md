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

## GNOME Wayland setup (one-time, no X11 switch needed)

Stock GNOME Wayland exposes no permission-free pointer API, so install the
companion extension once:

```bash
mkdir -p ~/.local/share/gnome-shell/extensions
cp -r packaging/gnome-extension/mousecoords@mouse-coords.github.io \
  ~/.local/share/gnome-shell/extensions/
# log out and back in (GNOME only loads new extensions at login), then:
gnome-extensions enable mousecoords@mouse-coords.github.io
```

Verify (move the mouse between calls — the numbers must change):

```bash
cargo run --example pos
cargo run --example pos -- --loop  # poll every 200ms
```

Without the extension (or the wdotool bridge, GNOME 45–48), the crate falls
back to Shell `Eval`, which needs unsafe mode since GNOME 41 (`Alt+F2` →
`lg` → `global.context.unsafe_mode = true`, valid until logout). If neither
is available it returns an explanatory error instead of a frozen coordinate.

## Hyprland (no setup)

Hyprland exposes the pointer through its IPC socket, so there is nothing to
install. The crate locates
`$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock` (falling back
to `/tmp/hypr/...`) and asks for `cursorpos` over it.

```bash
cargo run --example pos
```

## How it works (Linux)

- **Hyprland Wayland**: native IPC socket (`cursorpos`). No extension needed.
- **KDE Plasma Wayland** (e.g. Fedora KDE): KWin scripting over D-Bus
  (`workspace.cursorPos` + `callDBus` callback, same technique as
  `kdotool`/`wdotool`). It is the default backend when
  `XDG_SESSION_TYPE=wayland` + KDE desktop. Accurate, follows the mouse.
- **GNOME Wayland** (no X11 switch needed — probe order, first win):
  1. Companion Shell extension `GetPointerPosition()`: `org.mousecoords.Bridge`
     (shipped in `packaging/gnome-extension/`, GNOME 45–50) or
     `org.wdotool.GnomeShellBridge` (wdotool extension, GNOME 45–48).
  2. Shell `Eval("global.get_pointer()")` — needs unsafe mode since GNOME 41
     (Looking Glass via `Alt+F2` → `lg` → `global.context.unsafe_mode = true`,
     valid until logout; or the "Unsafe mode menu" extension).
  - Stock GNOME has no permission-free pointer API (GNOME rejected
    layer-shell / virtual-pointer, legacy AT-SPI sniffing is X11-only,
    libei/RemoteDesktop portal is input-only). If neither backend is
    available the crate returns an explanatory error instead of a frozen
    coordinate. See `packaging/gnome-extension/README.md` for install steps.
- **Plain X11**: `XQueryPointer` on the root window via `x11rb`.
- **Wayland + XWayland without a native backend**: `XQueryPointer` returns a
  **frozen** position (often the screen center, e.g. `x=640 y=400` on
  1280x800 — it only updates over X11 windows). The crate **does not**
  return this value; it returns `Error::Unsupported` instead.

## Other platforms

- **Windows**: `GetCursorPos`.
- **macOS**: `CGEventCreate` + `CGEventGetLocation`.
