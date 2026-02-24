```
╔═══════════════════════════════════════════════════════════╗
║   ██████╗ ███████╗███╗   ███╗██╗   ██╗██╗  ██╗            ║
║   ██╔══██╗██╔════╝████╗ ████║██║   ██║╚██╗██╔╝            ║
║   ██████╔╝███████╗██╔████╔██║██║   ██║ ╚███╔╝             ║
║   ██╔═══╝ ╚════██║██║╚██╔╝██║██║   ██║ ██╔██╗             ║
║   ██║     ███████║██║ ╚═╝ ██║╚██████╔╝██╔╝ ██╗            ║
║   ╚═╝     ╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝            ║
║     Born in PowerShell. Made in Rust. 🦀                 ║
║          Terminal Multiplexer for Windows                 ║
╚═══════════════════════════════════════════════════════════╝
```

<p align="center">
  <strong>The native Windows tmux. Born in PowerShell, made in Rust.</strong><br/>
  Full mouse support · tmux themes · tmux config · 76 commands · blazing fast
</p>

<p align="center">
  <a href="#installation">Install</a> ·
  <a href="#usage">Usage</a> ·
  <a href="#key-bindings">Keys</a> ·
  <a href="#configuration">Config</a> ·
  <a href="#performance">Performance</a> ·
  <a href="#tmux-compatibility">Compatibility</a>
</p>

---

# psmux

**The real tmux for Windows.** Not a port, not a wrapper, not a workaround.

psmux is a **native Windows terminal multiplexer** built from the ground up in Rust. It uses Windows ConPTY directly, speaks the tmux command language, reads your `.tmux.conf`, and supports tmux themes. All without WSL, Cygwin, or MSYS2.

> 💡 **Tip:** psmux ships with `tmux` and `pmux` aliases. Just type `tmux` and it works!

## Installation

### Using WinGet

```powershell
winget install psmux
```

### Using Cargo

```powershell
cargo install psmux
```

This installs `psmux`, `pmux`, and `tmux` binaries to your Cargo bin directory.

### Using Scoop

```powershell
scoop bucket add psmux https://github.com/marlocarlo/scoop-psmux
scoop install psmux
```

### Using Chocolatey

```powershell
choco install psmux
```

### From GitHub Releases

Download the latest `.zip` from [GitHub Releases](https://github.com/marlocarlo/psmux/releases) and add to your PATH.

### From Source

```powershell
git clone https://github.com/marlocarlo/psmux.git
cd psmux
cargo build --release
```

Built binaries:

```text
target\release\psmux.exe
target\release\pmux.exe
target\release\tmux.exe
```

### Requirements

- Windows 10 or Windows 11
- **PowerShell 7+** (recommended) or cmd.exe
  - Download PowerShell: `winget install --id Microsoft.PowerShell`
  - Or visit: https://aka.ms/powershell

## Why psmux?

If you've used tmux on Linux/macOS and wished you had something like it on Windows, **this is it**.

| | psmux | Windows Terminal tabs | WSL + tmux |
|---|:---:|:---:|:---:|
| Session persist (detach/reattach) | ✅ | ❌ | ⚠️ WSL only |
| Synchronized panes | ✅ | ❌ | ✅ |
| tmux keybindings | ✅ | ❌ | ✅ |
| Reads `.tmux.conf` | ✅ | ❌ | ✅ |
| tmux theme support | ✅ | ❌ | ✅ |
| Native Windows shells | ✅ | ✅ | ❌ |
| Full mouse support | ✅ | ✅ | ⚠️ Partial |
| Zero dependencies | ✅ | ✅ | ❌ (needs WSL) |
| Scriptable (76 commands) | ✅ | ❌ | ✅ |

![psmux in action - monitoring system info](psmux_sysinfo.gif)

### Highlights

- 🦠 **Made in Rust** : opt-level 3, full LTO, single codegen unit. Maximum performance.
- 🖱️ **Full mouse support** : click panes, drag-resize borders, scroll, click tabs, select text, right-click copy
- 🎨 **tmux theme support** : 16 named colors + 256 indexed + 24-bit true color (`#RRGGBB`), 14 style options
- 📋 **Reads your `.tmux.conf`** : drop-in config compatibility, zero learning curve
- ⚡ **Blazing fast startup** : sub-100ms session creation, near-zero overhead over shell startup
- 🔌 **76 tmux-compatible commands** : `bind-key`, `set-option`, `if-shell`, `run-shell`, hooks, and more
- 🪟 **Windows-native** : ConPTY, Win32 API, works with PowerShell, cmd, bash, WSL, nushell
- 📦 **Single binary, no dependencies** : install via `cargo`, `winget`, `scoop`, or `choco`

## Features

### Terminal Multiplexing
- Split panes horizontally (`Prefix + %`) and vertically (`Prefix + "`)
- Multiple windows with clickable status-bar tabs
- Session management: detach (`Prefix + d`) and reattach from anywhere
- 5 layouts: even-horizontal, even-vertical, main-horizontal, main-vertical, tiled

### Full Mouse Support
- **Click** any pane to focus it, input goes to the right shell
- **Drag** pane borders to resize splits interactively
- **Click** status-bar tabs to switch windows
- **Scroll wheel** in any pane, scrolls that pane's output
- **Drag-select** text to copy to clipboard
- **Right-click** to paste or copy selection
- **VT mouse forwarding** : apps like vim, htop, and midnight commander get full mouse events
- **3-layer mouse injection** : VT protocol, VT bridge (for WSL/SSH), and native Win32 MOUSE_EVENT

### tmux Theme & Style Support
- **14 customizable style options** : status bar, pane borders, messages, copy-mode highlights, popups
- **Full color spectrum** : 16 named colors, 256 indexed (`colour0`–`colour255`), 24-bit true color (`#RRGGBB`)
- **Text attributes** : bold, dim, italic, underline, blink, reverse, strikethrough, and more
- **Status bar** : fully customizable left/right content with format variables
- **Window tab styling** : separate styles for active, inactive, activity, bell, and last-used tabs
- Compatible with existing tmux theme configs

### Copy Mode (Vim Keybindings)
- **53 vi-style key bindings** : motions, selections, search, text objects
- Visual, line, and **rectangle selection** modes (`v`, `V`, `Ctrl+v`)
- `/` and `?` search with `n`/`N` navigation
- `f`/`F`/`t`/`T` character find, `%` bracket matching, `{`/`}` paragraph jump
- Named registers (`"a`–`"z`), count prefixes, word/WORD variants
- Mouse drag-select copies to Windows clipboard on release

### Format Engine
- **126+ tmux-compatible format variables** across sessions, windows, panes, cursor, client, and server
- Conditionals (`#{?cond,true,false}`), comparisons, boolean logic
- Regex substitution (`#{s/pat/rep/:var}`), string manipulation
- Loop iteration (`#{W:fmt}`, `#{P:fmt}`, `#{S:fmt}`) over windows, panes, sessions
- Truncation, padding, basename, dirname, strftime, shell quoting

### Scripting & Automation
- **76 tmux-compatible commands** : everything you need for automation
- `send-keys`, `capture-pane`, `pipe-pane` for CI/CD and DevOps workflows
- `if-shell` and `run-shell` for conditional config logic
- **15+ event hooks** : `after-new-window`, `after-split-window`, `client-attached`, etc.
- Paste buffers, named registers, `display-message` with format variables

### Multi-Shell Support
- **PowerShell 7** (default), PowerShell 5, cmd.exe
- **Git Bash**, WSL, nushell, and any Windows executable
- Sets `TERM=xterm-256color`, `COLORTERM=truecolor` automatically
- Sets `TMUX` and `TMUX_PANE` env vars for tmux-aware tool compatibility

![psmux windows and panes](psmux_windows.gif)

## Performance

psmux is built for speed. The Rust release binary is compiled with **opt-level 3**, **full LTO**, and **single codegen unit**. Every cycle counts.

| Metric | psmux | Notes |
|--------|-------|-------|
| **Session creation** | **< 100ms** | Time for `new-session -d` to return |
| **New window** | **< 80ms** | Overhead on top of shell startup |
| **New pane (split)** | **< 80ms** | Same as window, cached shell resolution |
| **Startup to prompt** | **~shell launch time** | psmux adds near-zero overhead; bottleneck is your shell |
| **15+ windows** | ✅ Stable | Stress-tested with 15+ rapid windows, 18+ panes, 5 concurrent sessions |
| **Rapid fire creates** | ✅ No hangs | Burst-create windows/panes without delays or orphaned processes |

### How it's fast

- **Lazy pane resize** : only the active window's panes are resized. Background windows resize on-demand when switched to, avoiding O(n) ConPTY syscalls
- **Cached shell resolution** : `which` PATH lookups are cached with `OnceLock`, not repeated per spawn
- **10ms polling** : client-server discovery uses tight 10ms polling for sub-100ms session attach
- **Early port-file write** : server writes its discovery file *before* spawning the first shell, so the client connects instantly
- **8KB reader buffers** : small buffer size minimizes mutex contention across pane reader threads

> **Note:** The primary startup bottleneck is your shell (PowerShell 7 takes ~400-1000ms to display a prompt). psmux itself adds < 100ms of overhead. For faster shells like `cmd.exe` or `nushell`, total startup is near-instant.

## tmux Compatibility

psmux is the most tmux-compatible terminal multiplexer on Windows:

| Feature | Support |
|---------|---------|
| Commands | **76** tmux commands implemented |
| Format variables | **126+** variables with full modifier support |
| Config file | Reads `~/.tmux.conf` directly |
| Key bindings | `bind-key`/`unbind-key` with key tables |
| Hooks | 15+ event hooks (`after-new-window`, etc.) |
| Status bar | Full format engine with conditionals and loops |
| Themes | 14 style options, 24-bit color, text attributes |
| Layouts | 5 layouts (even-h, even-v, main-h, main-v, tiled) |
| Copy mode | 53 vim keybindings, search, registers |
| Targets | `session:window.pane`, `%id`, `@id` syntax |
| `if-shell` / `run-shell` | ✅ Conditional config logic |
| Paste buffers | ✅ Full buffer management |

**Your existing `.tmux.conf` works.** psmux reads it automatically. Just install and go.

## Usage

Use `psmux`, `pmux`, or `tmux`, they're identical:

```powershell
# Start a new session
psmux
pmux
tmux

# Start a named session
psmux new-session -s work
tmux new-session -s work

# List sessions
psmux ls
tmux ls

# Attach to a session
psmux attach -t work
tmux attach -t work

# Show help
psmux --help
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
| `Prefix + &` | Kill current window |
| `Prefix + z` | Toggle pane zoom |
| `Prefix + n` | Next window |
| `Prefix + p` | Previous window |
| `Prefix + 0-9` | Select window by number |
| `Prefix + d` | Detach from session |
| `Prefix + ,` | Rename current window |
| `Prefix + t` | Show clock |
| `Prefix + s` | Session chooser/switcher |
| `Prefix + o` | Select next pane |
| `Prefix + w` | Window/pane chooser |
| `Prefix + [` | Enter copy/scroll mode |
| `Prefix + {` | Swap pane up |
| `Prefix + ]` | Paste from buffer |
| `Prefix + q` | Display pane numbers |
| `Prefix + Arrow` | Navigate between panes |
| `Ctrl+q` | Quit |

### Copy/Scroll Mode

Enter copy mode with `Prefix + [` to scroll through terminal history with **53 vim-style keybindings**:

| Key | Action |
|-----|--------|
| `↑` / `k` | Move cursor / scroll up |
| `↓` / `j` | Move cursor / scroll down |
| `h` / `l` | Move cursor left / right |
| `w` / `b` / `e` | Next word / prev word / end of word |
| `W` / `B` / `E` | WORD variants (whitespace-delimited) |
| `0` / `$` / `^` | Start / end / first non-blank of line |
| `g` / `G` | Jump to top / bottom of scrollback |
| `H` / `M` / `L` | Top / middle / bottom of screen |
| `Ctrl+u` / `Ctrl+d` | Scroll half page up / down |
| `Ctrl+b` / `Ctrl+f` | Scroll full page up / down |
| `f{char}` / `F{char}` | Find char forward / backward |
| `t{char}` / `T{char}` | Till char forward / backward |
| `%` | Jump to matching bracket |
| `{` / `}` | Previous / next paragraph |
| `/` / `?` | Search forward / backward |
| `n` / `N` | Next / previous match |
| `v` | Begin selection |
| `V` | Line selection |
| `Ctrl+v` | Rectangle selection |
| `o` | Swap selection ends |
| `y` / `Enter` | Yank (copy) selection |
| `D` | Copy to end of line |
| `"a`–`"z` | Named registers |
| `1`–`9` | Count prefix for motions |
| `Mouse drag` | Select text → copies to clipboard on release |
| `Esc` / `q` | Exit copy mode |

When in copy mode:
- The pane border turns **yellow** 
- `[copy mode]` appears in the title
- A scroll position indicator shows in the top-right corner
- Mouse selection in copy mode is copied to the Windows clipboard on release

## Scripting & Automation

psmux supports tmux-compatible commands for scripting and automation:

### Window & Pane Control

```powershell
# Create a new window
psmux new-window

# Split panes
psmux split-window -v          # Split vertically (top/bottom)
psmux split-window -h          # Split horizontally (side by side)

# Navigate panes
psmux select-pane -U           # Select pane above
psmux select-pane -D           # Select pane below
psmux select-pane -L           # Select pane to the left
psmux select-pane -R           # Select pane to the right

# Navigate windows
psmux select-window -t 1       # Select window by index (default base-index is 1)
psmux next-window              # Go to next window
psmux previous-window          # Go to previous window
psmux last-window              # Go to last active window

# Kill panes and windows
psmux kill-pane
psmux kill-window
psmux kill-session
```

### Sending Keys

```powershell
# Send text directly
psmux send-keys "ls -la" Enter

# Send keys literally (no parsing)
psmux send-keys -l "literal text"

# Special keys supported:
# Enter, Tab, Escape, Space, Backspace
# Up, Down, Left, Right, Home, End
# PageUp, PageDown, Delete, Insert
# F1-F12, C-a through C-z (Ctrl+key)
```

### Pane Information

```powershell
# List all panes in current window
psmux list-panes

# List all windows
psmux list-windows

# Capture pane content
psmux capture-pane

# Display formatted message with variables
psmux display-message "#S:#I:#W"   # Session:Window Index:Window Name
```

### Paste Buffers

```powershell
# Set paste buffer content
psmux set-buffer "text to paste"

# Paste buffer to active pane
psmux paste-buffer

# List all buffers
psmux list-buffers

# Show buffer content
psmux show-buffer

# Delete buffer
psmux delete-buffer
```

### Pane Layout

```powershell
# Resize panes
psmux resize-pane -U 5         # Resize up by 5
psmux resize-pane -D 5         # Resize down by 5
psmux resize-pane -L 10        # Resize left by 10
psmux resize-pane -R 10        # Resize right by 10

# Swap panes
psmux swap-pane -U             # Swap with pane above
psmux swap-pane -D             # Swap with pane below

# Rotate panes in window
psmux rotate-window

# Toggle pane zoom
psmux zoom-pane
```

### Session Management

```powershell
# Check if session exists (exit code 0 = exists)
psmux has-session -t mysession

# Rename session
psmux rename-session newname

# Respawn pane (restart shell)
psmux respawn-pane
```

### Format Variables

The `display-message` command supports these variables:

| Variable | Description |
|----------|-------------|
| `#S` | Session name |
| `#I` | Window index |
| `#W` | Window name |
| `#P` | Pane ID |
| `#T` | Pane title |
| `#H` | Hostname |

### Advanced Commands

```powershell
# Discover supported commands
psmux list-commands

# Server/session management
psmux kill-server
psmux list-clients
psmux switch-client -t other-session

# Config at runtime
psmux source-file ~/.psmux.conf
psmux show-options
psmux set-option -g status-left "[#S]"

# Layout/history/stream control
psmux next-layout
psmux previous-layout
psmux clear-history
psmux pipe-pane -o "cat > pane.log"

# Hooks
psmux set-hook -g after-new-window "display-message created"
psmux show-hooks
```

### Target Syntax (`-t`)

psmux supports tmux-style targets:

```powershell
# window by index in session
psmux select-window -t work:2

# specific pane by index
psmux send-keys -t work:2.1 "echo hi" Enter

# pane by pane id
psmux send-keys -t %3 "pwd" Enter

# window by window id
psmux select-window -t @4
```

## Configuration

psmux reads its config on startup from the **first file found** (in order):

1. `~/.psmux.conf`
2. `~/.psmuxrc`
3. `~/.tmux.conf`
4. `~/.config/psmux/psmux.conf`

Config syntax is **tmux-compatible**. Most `.tmux.conf` lines work as-is.

### Basic Config Example

Create `~/.psmux.conf`:

```tmux
# Change prefix key to Ctrl+a
set -g prefix C-a

# Enable mouse
set -g mouse on

# Window numbering base (default is 1)
set -g base-index 1

# Customize status bar
set -g status-left "[#S] "
set -g status-right "%H:%M %d-%b-%y"
set -g status-style "bg=green,fg=black"

# Cursor style: block, underline, or bar
set -g cursor-style bar
set -g cursor-blink on

# Scrollback history
set -g history-limit 5000

# Prediction dimming (disable for apps like Neovim)
set -g prediction-dimming off

# Key bindings
bind-key -T prefix h split-window -h
bind-key -T prefix v split-window -v
```

### Choosing a Shell

psmux launches **PowerShell 7 (pwsh)** by default. You can change this:

```tmux
# Use cmd.exe
set -g default-shell cmd

# Use PowerShell 5 (Windows built-in)
set -g default-shell powershell

# Use PowerShell 7 (explicit path)
set -g default-shell "C:/Program Files/PowerShell/7/pwsh.exe"

# Use Git Bash
set -g default-shell "C:/Program Files/Git/bin/bash.exe"

# Use Nushell
set -g default-shell nu

# Use Windows Subsystem for Linux (via wsl.exe)
set -g default-shell wsl
```

You can also launch a window with a specific command without changing the default:

```powershell
psmux new-window -- cmd /K echo hello
psmux new-session -s py -- python
psmux split-window -- "C:/Program Files/Git/bin/bash.exe"
```

### All Set Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `prefix` | Key | `C-b` | Prefix key |
| `base-index` | Int | `1` | First window number |
| `pane-base-index` | Int | `0` | First pane number |
| `escape-time` | Int | `500` | Escape delay (ms) |
| `repeat-time` | Int | `500` | Repeat key timeout (ms) |
| `history-limit` | Int | `2000` | Scrollback lines per pane |
| `display-time` | Int | `750` | Message display time (ms) |
| `display-panes-time` | Int | `1000` | Pane overlay time (ms) |
| `status-interval` | Int | `15` | Status refresh (seconds) |
| `mouse` | Bool | `on` | Mouse support |
| `status` | Bool | `on` | Show status bar |
| `status-position` | Str | `bottom` | `top` or `bottom` |
| `focus-events` | Bool | `off` | Pass focus events to apps |
| `mode-keys` | Str | `emacs` | `vi` or `emacs` |
| `renumber-windows` | Bool | `off` | Auto-renumber windows on close |
| `automatic-rename` | Bool | `on` | Rename windows from foreground process |
| `monitor-activity` | Bool | `off` | Flag windows with new output |
| `monitor-silence` | Int | `0` | Seconds before silence flag (0=off) |
| `synchronize-panes` | Bool | `off` | Send input to all panes |
| `remain-on-exit` | Bool | `off` | Keep panes after process exits |
| `aggressive-resize` | Bool | `off` | Resize to smallest client |
| `set-titles` | Bool | `off` | Update terminal title |
| `set-titles-string` | Str | | Terminal title format |
| `default-shell` | Str | `pwsh` | Shell to launch |
| `default-command` | Str | | Alias for default-shell |
| `word-separators` | Str | `" -_@"` | Copy-mode word delimiters |
| `prediction-dimming` | Bool | `off` | Dim predictive text |
| `cursor-style` | Str | | `block`, `underline`, or `bar` |
| `cursor-blink` | Bool | `off` | Cursor blinking |
| `bell-action` | Str | `any` | `any`, `none`, `current`, `other` |
| `visual-bell` | Bool | `off` | Visual bell indicator |
| `status-left` | Str | `[#S] ` | Left status bar content |
| `status-right` | Str | | Right status bar content |
| `status-style` | Str | `bg=green,fg=black` | Status bar style |
| `status-left-style` | Str | | Left status style |
| `status-right-style` | Str | | Right status style |
| `status-justify` | Str | `left` | Tab alignment: `left`, `centre`, `right` |
| `message-style` | Str | `bg=yellow,fg=black` | Message style |
| `message-command-style` | Str | `bg=black,fg=yellow` | Command prompt style |
| `mode-style` | Str | `bg=yellow,fg=black` | Copy-mode highlight |
| `pane-border-style` | Str | | Inactive border style |
| `pane-active-border-style` | Str | `fg=green` | Active border style |
| `window-status-format` | Str | `#I:#W#F` | Inactive tab format |
| `window-status-current-format` | Str | `#I:#W#F` | Active tab format |
| `window-status-separator` | Str | `" "` | Tab separator |
| `window-status-style` | Str | | Inactive tab style |
| `window-status-current-style` | Str | | Active tab style |
| `window-status-activity-style` | Str | `reverse` | Activity tab style |
| `window-status-bell-style` | Str | `reverse` | Bell tab style |
| `window-status-last-style` | Str | | Last-active tab style |

Style format: `"fg=colour,bg=colour,bold,dim,underscore,italics,reverse"`

Colours: `default`, `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `colour0`–`colour255`, `#RRGGBB`

### Environment Variables

```powershell
# Default session name used when not explicitly provided
$env:PSMUX_DEFAULT_SESSION = "work"

# Enable prediction dimming (off by default; dims predictive/speculative text)
$env:PSMUX_DIM_PREDICTIONS = "1"

# These are set INSIDE psmux panes (tmux-compatible):
# TMUX       - socket path and server info
# TMUX_PANE  - current pane ID (%0, %1, etc.)
```

### Prediction Dimming

Prediction dimming is off by default. If you want psmux to dim predictive/speculative text (e.g. shell autosuggestions), you can enable it in `~/.psmux.conf`:

```tmux
set -g prediction-dimming on
```

You can also enable it for the current shell only:

```powershell
$env:PSMUX_DIM_PREDICTIONS = "1"
psmux
```

To make it persistent for new shells:

```powershell
setx PSMUX_DIM_PREDICTIONS 1
```

## License

MIT

---

## About psmux

**psmux** (PowerShell Multiplexer) is a terminal multiplexer **born in PowerShell, made in Rust**, built from scratch for Windows. It's not a tmux port. It's not a compatibility layer. It's a native Windows application that speaks fluent tmux.

psmux exists because Windows developers deserve the same terminal multiplexing experience that Linux and macOS users have enjoyed for decades, without being forced into WSL, Cygwin, or MSYS2.

### What makes psmux different

- **Native Windows binary** : uses ConPTY directly, no POSIX translation layer
- **Full tmux command compatibility** : 76 commands, 126+ format variables, reads `.tmux.conf`
- **Full mouse support** : 3-layer injection system handles native shells, TUI apps, and WSL/SSH seamlessly
- **Theme support** : bring your tmux themes, they work here
- **Performance-optimized Rust** : opt-level 3, LTO, cached everything, sub-100ms startup
- **Single binary** : `psmux.exe` (also installs as `pmux.exe` and `tmux.exe`), no runtime dependencies

### Star History

If psmux helps your Windows workflow, consider giving it a ⭐ on GitHub. It helps others find it!

### Contributing

Contributions are welcome! Whether it's:
- 🐛 Bug reports and feature requests via [GitHub Issues](https://github.com/marlocarlo/psmux/issues)
- 💻 Pull requests for fixes and features
- 📖 Documentation improvements
- 🧪 Test scripts and compatibility reports

### Keywords

terminal multiplexer, tmux for windows, tmux alternative, tmux windows, windows terminal multiplexer, powershell multiplexer, split terminal windows, multiple terminals, terminal tabs, pane splitting, session management, windows terminal, powershell terminal, cmd terminal, rust terminal, console multiplexer, terminal emulator, windows console, cli tool, command line, devtools, developer tools, productivity, windows 10, windows 11, psmux, pmux, conpty, tmux themes, tmux config, mouse support, copy mode, vim keybindings, rust windows, native windows tmux, tmux clone, terminal panes, powerline, powershell 7, pwsh, winget, scoop, chocolatey, cargo install

### Related Projects

- [tmux](https://github.com/tmux/tmux) : The original terminal multiplexer for Unix/Linux/macOS
- [Windows Terminal](https://github.com/microsoft/terminal) : Microsoft's modern terminal for Windows
- [PowerShell](https://github.com/PowerShell/PowerShell) : Cross-platform PowerShell

### FAQ

**Q: Is psmux cross-platform?**  
A: No. psmux is built exclusively for Windows using the Windows ConPTY API. For Linux/macOS, use tmux. psmux is the Windows counterpart.

**Q: Does psmux work with Windows Terminal?**  
A: Yes! psmux works great with Windows Terminal, PowerShell, cmd.exe, ConEmu, and other Windows terminal emulators.

**Q: Why use psmux instead of Windows Terminal tabs?**  
A: psmux offers session persistence (detach/reattach), synchronized input to multiple panes, full tmux command scripting, hooks, format engine, and tmux-compatible keybindings. Windows Terminal tabs can't do any of that.

**Q: Can I use my existing `.tmux.conf`?**  
A: Yes! psmux reads `~/.tmux.conf` automatically. Most tmux config options, key bindings, and style settings work as-is.

**Q: Can I use tmux themes?**  
A: Yes. psmux supports 14 style options with 24-bit true color, 256 indexed colors, and text attributes (bold, italic, dim, etc.). Most tmux theme configs are compatible.

**Q: Can I use tmux commands with psmux?**  
A: Yes! psmux includes a `tmux` alias. Commands like `tmux new-session`, `tmux attach`, `tmux ls`, `tmux split-window` all work. 76 commands in total.

**Q: How fast is psmux?**  
A: Session creation takes < 100ms. New windows/panes add < 80ms overhead. The bottleneck is your shell's startup time, not psmux. Compiled with opt-level 3 and full LTO.

**Q: Does psmux support mouse?**  
A: Full mouse support: click to focus panes, drag to resize borders, scroll wheel, click status-bar tabs, drag-select text, right-click copy. Plus VT mouse forwarding for TUI apps like vim, htop, and midnight commander.

**Q: What shells does psmux support?**  
A: PowerShell 7 (default), PowerShell 5, cmd.exe, Git Bash, WSL, nushell, and any Windows executable. Change with `set -g default-shell <shell>`.

**Q: Is it stable for daily use?**  
A: Yes. psmux is stress-tested with 15+ rapid windows, 18+ concurrent panes, 5 concurrent sessions, kill+recreate cycles, and sustained load, all with zero hangs or resource leaks.

---

<p align="center">
  Made with ❤️ for PowerShell using Rust 🦀
</p>
