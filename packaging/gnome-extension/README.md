# GNOME Shell extension (Wayland workaround, no X11 switch needed)

Stock GNOME Wayland exposes no permission-free global-pointer API, so
`mouse_coords::get_position()` reads it through this tiny bridge, which calls
`global.get_pointer()` inside the compositor and answers
`GetPointerPosition()` on the session bus
(`org.mousecoords.Bridge` / `/org/mousecoords/Bridge`).

The crate also understands the wdotool bridge
(`org.wdotool.GnomeShellBridge`, GNOME 45–48) and Shell `Eval` with unsafe
mode enabled — the extension below is only needed if neither applies.

## Install

```sh
cp -r packaging/gnome-extension/mousecoords@mouse-coords.github.io \
  ~/.local/share/gnome-shell/extensions/
# log out and back in (a reload is not enough for a new extension), then:
gnome-extensions enable mousecoords@mouse-coords.github.io
```

Verify:

```sh
gdbus call --session --dest org.mousecoords.Bridge \
  --object-path /org/mousecoords/Bridge \
  --method org.mousecoords.Bridge.GetPointerPosition
# e.g. (int32 123, int32 456) — move the mouse and call again
```

No-install alternative for one session: Looking Glass (`Alt+F2`, type `lg`),
then `global.context.unsafe_mode = true`, and retry — Shell `Eval` works
until logout.
