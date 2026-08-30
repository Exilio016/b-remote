# Samsung TV ncurses Remote Controller

`b-controller` is a terminal-based remote controller for Samsung Smart TVs.

The application uses `ncurses` to provide an interactive terminal interface and connects to the TV through Samsung's WebSocket API. Keyboard and mouse input received by the ncurses interface is translated into Samsung TV remote-control commands.

The TV is located through the mDNS hostname `Samsung.local`

## Features

- Terminal user interface powered by ncurses
- Keyboard input support
- Mouse input support where supported by the terminal
- Automatic TV discovery through `Samsung.local`
- WebSocket communication with a Samsung Smart TV
- Secure WebSocket support through OpenSSL

## Requirements

- C++ compiler with C++17 support
- CMake
- Boost with the following components:
  - Headers
  - URL
  - JSON
- OpenSSL
- Curses/ncurses
- Samsung Smart TV with network access
- An mDNS-capable local network

The computer running the application and the TV must be connected to the same local network.

## Installing Dependencies

### Debian or Ubuntu

```bash
sudo apt update

sudo apt install \
  build-essential \
  cmake \
  pkg-config \
  libboost-all-dev \
  libssl-dev \
  libncurses-dev
```

### Fedora
```bash
sudo dnf install \
  gcc-c++ \
  cmake \
  pkgconf-pkg-config \
  boost-devel \
  openssl-devel \
  ncurses-devel
```
### Arch Linux
```bash
sudo pacman -S \
  base-devel \
  cmake \
  pkgconf \
  boost \
  openssl \
  ncurses
```

## Building
Clone this repo:
```bash
git clone https://github.com/Exilio016/bController
cd bController
```
Configure the project:
```bash
cmake -B build
```
Build the project:
```bash
cmake --build build
```
The final executable will be localted at `build/b-controller`.

## Controls
After starting the application, the remote controller is rendered directly in the terminal window.

You can interact with the controller in either of the following ways:

- Click the on-screen buttons using the mouse.
- Use the keyboard shortcuts listed below.

| Input | Action |
|:------|:-------|
| Arrow keys | Navigate |
| Enter | Select or confirm |
| Esc | Go back |
| `q` | Quit |
| `m` | Toggle mute |
| `h` | Home |
| `+` / `-` | Increase or decrease the volume |

Mouse support depends on the terminal emulator and the environment in which the application is running.
