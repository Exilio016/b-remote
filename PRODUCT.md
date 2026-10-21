# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Native Rust desktop application with dual rendering backends:
- **GUI**: `raylib` (via `sola-raylib`) — native windowed application
- **TUI**: `crosscurses` — runs in any terminal, works over SSH
- **Network**: `tungstenite` + `native-tls` for secure WebSocket to Samsung TV
- **Assets**: Embedded at compile time via `build.rs` from `assets/`
- **Targets**: Linux, macOS, Windows (cross-platform binary)

## Users

**Primary**: Someone who lost or broke their physical Samsung TV remote and needs a functional replacement on their computer.

**Situation**: At their desk or on the same network as the TV, wanting immediate control without phone apps or buying a new remote.

**Job**: Control Samsung TV (power, volume, channels, navigation, playback, home/menu) with keyboard shortcuts or mouse, locally or remotely via SSH.

## Product Purpose

A Samsung TV remote controller that runs natively on desktop (Linux/macOS/Windows) with two interfaces: a graphical window (raylib) and a terminal UI (crosscurses). Connects via WebSocket to the TV at `Samsung.local` using mDNS. Success means: connects reliably, maps every physical remote button to a keyboard shortcut, works both locally and over SSH, starts instantly, uses minimal resources.

## Positioning

The only Samsung TV remote that is a **native cross-platform Rust binary** with **dual TUI + GUI interfaces** — works identically on a local desktop and over an SSH session, with zero Electron/browser overhead and instant startup.

## Operating Context

- TV and computer on same local network
- mDNS resolution working (`Samsung.local` resolves)
- TV has "Remote Control" enabled in Settings → General → External Device Manager
- First run prompts on TV for permission
- User operates via keyboard (primary) or mouse (secondary)
- Typical session: launch, control TV, quit — no persistent session needed

## Capabilities and Constraints

**Confirmed functionality**:
- 17-button layout: Power, VolUp, VolDown, Mute, ChUp, ChDown, ChList, Up, Down, Left, Right, Enter, Back, Play, Pause, Home, Menu
- Keyboard shortcuts mapped 1:1 to each button (see Controls table in README)
- Mouse clicks work on both TUI and GUI where supported
- mDNS discovery at `Samsung.local`
- Secure WebSocket (WSS) via native-tls
- Asset embedding at compile time (14 PNG icons in `assets/`)

**Explicitly undecided**:
- Whether to add more buttons (e.g., number pad, color keys, source, apps)
- Whether to support multiple TVs / TV selection UI
- Whether to persist last-known TV IP/token for faster reconnect
- Whether to add a pairing/onboarding flow for first-time users

**Constraints to preserve**:
- Keyboard shortcuts mapped 1:1 to buttons (muscle memory)
- mDNS discovery at `Samsung.local` (zero-config)
- Cross-platform Rust binary (Linux/macOS/Windows)
- Raylib and terminal interfaces only (no web/electron/tauri)
- 17-button grid layout at fixed positions (7×9 grid)
- 14 existing PNG assets in `assets/`

## Brand Commitments

- Voice: Technical, direct, no marketing fluff
- Identity: Developer-tool aesthetic — functional, keyboard-first, minimal
- No external brand assets beyond the 14 PNG icons in `assets/`

## Evidence on Hand

- Working Rust codebase (compiles, runs, connects to TV)
- 14 PNG icons in `assets/` (arrow-up, arrow-down, circle-arrow-*, circle-power, circle, house, pause, play, settings, tv, undo-2, volume-x)
- README with complete controls table and build/run instructions
- No testimonials, case studies, press, or fabricated metrics — future work must not invent these

## Product Principles

1. **Native first**: Rust binary, no runtime baggage, instant startup, minimal RAM.
2. **Dual interface parity**: TUI and GUI expose identical controls, shortcuts, and behavior.
3. **Keyboard as primary**: Every button has a shortcut; mouse is secondary.
4. **Zero config**: mDNS + `Samsung.local` — works without IP entry or setup screens.
5. **Honest constraints**: No fake features, no invented testimonials, no promised platforms.

## Accessibility & Inclusion

- Terminal UI provides screen-reader compatible control over SSH
- Keyboard-only operation fully supported (no mouse required)
- High contrast in TUI is terminal-dependent; GUI uses simple colored shapes
- No product-specific accessibility standard established beyond native platform defaults
