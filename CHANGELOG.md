# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-09-10

### Added

- Hyprland (Wayland) backend: reads the pointer over the Hyprland IPC socket
  (`cursorpos`), looking in
  `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock` with a
  `/tmp/hypr/...` fallback and an instance-directory scan when the signature is
  unset. No extension or extra dependency required.

### Changed

- `Error::Unsupported` message now lists KWin, GNOME Shell Eval and Hyprland
  IPC as the attempted Wayland backends.

## [0.2.0] - 2026-09-07

### Added

- GNOME Wayland backend: companion Shell extension bridge
  (`org.mousecoords.Bridge` / `org.wdotool.GnomeShellBridge`) with a Shell
  `Eval("global.get_pointer()")` fallback.

### Changed

- Do not fall back to `XQueryPointer` on Wayland: it returns a frozen position
  (often the screen center). Return `Error::Unsupported` instead.

## [0.1.0] - 2026-09-05

### Added

- Initial release: KDE Plasma Wayland (KWin scripting over D-Bus), X11,
  Windows and macOS backends.
