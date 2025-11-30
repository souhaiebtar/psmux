# pmux

**A terminal multiplexer for Windows** — the tmux alternative you've been waiting for.

pmux brings tmux-style terminal multiplexing to Windows natively. No WSL, no Cygwin, no compromises. Built in Rust for Windows Terminal, PowerShell, and cmd.exe.

> 💡 **Bonus:** pmux also installs as `tmux`, so you can use your muscle memory!

## Why pmux?

If you've used tmux on Linux/macOS and wished you had something similar on Windows — this is it.

- **Windows-native** — Built specifically for Windows 10/11
- **Works everywhere** — Windows Terminal, PowerShell, cmd.exe, ConEmu, etc.
- **No dependencies** — Single binary, just works
- **tmux-compatible** — Same commands, same keybindings, zero learning curve
- **Use as `tmux`** — Installs both `pmux` and `tmux` commands

## Features

- Split panes horizontally and vertically
- Multiple windows with tabs
- Session management (attach/detach)
- Mouse support for resizing panes
- Copy mode with vim-like keybindings
- Synchronized input to multiple panes

## Requirements

- Windows 10 or Windows 11
- PowerShell 5.1+, PowerShell Core 7+, or cmd.exe

## Installation

### Using Cargo

```powershell
cargo install pmux
```

This installs both `pmux` and `tmux` commands.

### Using Scoop

```powershell
scoop bucket add extras
scoop install pmux
```

### Using Winget

```powershell
winget install marlocarlo.pmux
```

### Using Chocolatey

```powershell
choco install pmux
```

### From GitHub Releases

Download the latest `.zip` from [GitHub Releases](https://github.com/marlocarlo/pmux/releases) and add to your PATH.

### From Source

```powershell
git clone https://github.com/marlocarlo/pmux.git
cd pmux
cargo install --path .
```

## Usage

Use `pmux` or `tmux` — they're identical:

```powershell
# Start a new session
pmux
tmux

# Start a named session
pmux new-session -s work
tmux new-session -s work

# List sessions
pmux ls
tmux ls

# Attach to a session
pmux attach -t work
tmux attach -t work

# Show help
pmux --help
tmux --help
```

## Key Bindings

Default prefix: `Ctrl+b` (same as tmux)

| Key | Action |
|-----|--------|
| `Prefix + c` | Create new window |
| `Prefix + %` | Split pane left/right |
| `Prefix + "` | Split pane top/bottom |
| `Prefix + x` | Kill current pane |
| `Prefix + z` | Toggle pane zoom |
| `Prefix + n` | Next window |
| `Prefix + p` | Previous window |
| `Prefix + 0-9` | Select window by number |
| `Prefix + d` | Detach from session |
| `Prefix + ,` | Rename current window |
| `Prefix + w` | Window/pane chooser |
| `Prefix + [` | Enter copy mode |
| `Prefix + ]` | Paste from buffer |
| `Prefix + q` | Display pane numbers |
| `Prefix + Arrow` | Navigate between panes |
| `Ctrl+q` | Quit |

## Configuration

Create `~/.pmux.conf`:

```
# Change prefix key to Ctrl+a
set -g prefix C-a

# Enable mouse
set -g mouse on

# Customize status bar
set -g status-left "[#S]"
set -g status-right "%H:%M"

# Cursor style: block, underline, or bar
set -g cursor-style bar
set -g cursor-blink on
```

## License

MIT