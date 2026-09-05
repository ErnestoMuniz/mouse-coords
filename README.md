# mouse-coords

Pega a coordenada exata do mouse em Rust.

```rust
let pos = mouse_coords::get_position().unwrap();
println!("x={} y={}", pos.x, pos.y);
```

Ou rode o exemplo:

```bash
cargo run --example pos
cargo run --example pos -- --loop  # poll a cada 200ms, mova o mouse para ver mudar
```

## Como funciona (Linux)

- **KDE Plasma Wayland** (ex: Fedora KDE): KWin scripting via D-Bus
  (`workspace.cursorPos` + `callDBus` de volta, mesma técnica do
  `kdotool`/`wdotool`). É o backend padrão quando
  `XDG_SESSION_TYPE=wayland` + desktop KDE. Exato, segue o mouse.
- **X11 puro**: `XQueryPointer` na root window via `x11rb`.
- **Wayland + XWayland sem KWin**: `XQueryPointer` retorna posição
  **congelada** (só atualiza sobre janelas X11) — por isso o KWin é
  preferido no KDE. Não use o X11 como referência no Wayland.

## Outras plataformas

- **Windows**: `GetCursorPos`.
- **macOS**: `CGEventCreate` + `CGEventGetLocation`.
