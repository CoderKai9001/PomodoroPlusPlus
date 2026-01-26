# 🍅 Pomodoro++

A beautiful terminal-based Pomodoro timer application built with Rust.

![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## ✨ Features

- **⏱️ Pomodoro Timer** - Configurable work/break durations with visual countdown
- **📊 Progress Bar** - Visual progress indicator for current session
- **🏷️ Tag System** - Organize sessions by custom tags (Work, Study, etc.)
- **📈 Statistics** - Weekly and monthly activity charts
- **📅 Heatmap** - GitHub-style activity heatmap (last 6 months)
- **🔔 Notifications** - Desktop notifications when sessions complete
- **🔊 Sound Alerts** - Audio notification on timer completion
- **💾 Persistence** - SQLite database stores all sessions and settings

## 📦 Installation

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Linux with `notify-send` and `paplay` for notifications/sounds

### Build from Source

```bash
git clone https://github.com/yourusername/PomodoroPlusPlus.git
cd PomodoroPlusPlus
cargo build --release
```

The binary will be at `./target/release/pomodoro-pp`

## 🚀 Usage

```bash
./target/release/pomodoro-pp
```

Or install globally:

```bash
cargo install --path .
pomodoro-pp
```

## ⌨️ Keyboard Controls

### Home Screen

| Key | Action |
|-----|--------|
| `Space` | Start/Pause timer |
| `r` | Reset timer |
| `t` / `↑↓` | Change tag |
| `+` / `n` | Add new tag |
| `-` | Delete selected tag |
| `w` / `W` | Adjust work duration ±1 min |
| `b` / `B` | Adjust break duration ±1 min |
| `s` | Statistics screen |
| `m` | Heatmap screen |
| `q` | Quit |

### Statistics Screen

| Key | Action |
|-----|--------|
| `Tab` | Toggle weekly/monthly view |
| `←` / `→` | Filter by tag |
| `h` | Home screen |
| `q` | Quit |

### Heatmap Screen

| Key | Action |
|-----|--------|
| `h` | Home screen |
| `s` | Statistics screen |
| `q` | Quit |

## 📁 Data Storage

Data is stored in:
- **Linux**: `~/.local/share/pomodoro++/pomodoro.db`

## 🔧 Configuration

Default settings (adjustable in-app):
- Work duration: 25 minutes
- Break duration: 5 minutes

Settings are persisted across sessions.

## 🎵 Sound Configuration

By default, the app plays `~/Music/sf/vieboom.mp3` when a session completes. You can place your preferred notification sound at this location.

## 🛠️ Tech Stack

- **[Ratatui](https://github.com/ratatui-org/ratatui)** - Terminal UI framework
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** - Cross-platform terminal manipulation
- **[Rusqlite](https://github.com/rusqlite/rusqlite)** - SQLite bindings
- **[Chrono](https://github.com/chronotope/chrono)** - Date/time handling

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
