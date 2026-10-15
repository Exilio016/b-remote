# bController

A Samsung TV remote controller written in Rust with both terminal (TUI) and graphical (raylib) interfaces.

## Features

- **Terminal UI (TUI)** powered by `crosscurses` — runs in any terminal
- **Graphical UI (GUI)** powered by `raylib` — native windowed application
- Keyboard and mouse input support
- Connects to Samsung Smart TVs via WebSocket API
- Secure WebSocket support through `native-tls`
- Asset embedding at compile time via `build.rs`

## Requirements

- Rust 1.70+ (2024 edition)
- Samsung Smart TV on the same local network
- TV must have "Remote Control" enabled in Settings → General → External Device Manager

## Building

```bash
git clone https://github.com/Exilio016/bController
cd bController
cargo build --release
```

The executable will be at `target/release/bController`.

## Running

### Terminal UI (TUI)
```bash
cargo run --release -- --tui
# or directly:
./target/release/bController --tui
```

### Graphical UI (GUI)
```bash
cargo run --release
# or directly:
./target/release/bController
```

## Controls

Both interfaces share the same button layout and keyboard shortcuts:

| Key | Action |
|:----|:-------|
| Arrow keys / `h` `j` `k` `l` | Navigate (Up/Down/Left/Right) |
| `Enter` / `Space` | Select / Confirm / Play-Pause |
| `Esc` / `b` | Back |
| `q` | Quit |
| `m` | Toggle mute |
| `h` | Home |
| `s` | Menu / Settings |
| `+` / `=` | Volume up |
| `-` / `_` | Volume down |
| `p` | Power |
| `g` | Channel list |

Mouse clicks work on both interfaces where supported.

## TV Discovery

The app connects to the TV at `Samsung.local` via mDNS. Ensure:

1. Your computer and TV are on the same network
2. mDNS resolution works (`ping Samsung.local` should resolve)
3. TV has "Remote Control" permission enabled for new devices (first run will prompt on TV)

## Project Structure

- `src/main.rs` — Entry point, button layout, renderer selection
- `src/remote.rs` — WebSocket communication with Samsung TV
- `src/renderer/term.rs` — Terminal UI using `crosscurses`
- `src/renderer/raylib.rs` — Graphical UI using `raylib`
- `src/renderer/mod.rs` — Shared `Button` struct and `Renderer` trait
- `build.rs` — Embeds assets from `assets/` into the binary
- `assets/` — PNG icons for buttons
