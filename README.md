# Seep

A fast, keyboard-driven terminal messaging client that brings WhatsApp, Discord, and iMessage into a single TUI interface.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Go](https://img.shields.io/badge/Go-00ADD8?style=flat&logo=go&logoColor=white)

## Features

- **Unified inbox** — view conversations across WhatsApp, Discord, and iMessage in one place
- **Keyboard driven** — navigate entirely with arrow keys and vim-style bindings
- **Platform color coding** — instantly identify which platform a conversation belongs to
- **Persistent sessions** — log in once, stay logged in
- **Async architecture** — non-blocking UI, messages load in the background
- **Lightweight** — runs entirely in your terminal, no Electron, no browser

## Screenshots

```
┌─────────────┬────────────────────────────────────┐
│ Chats       │ Messages                           │
│             │                                    │
│ > Alice     │ 10:00 Alice: Hey!                  │
│   Bob       │ 10:01 you: Hi Alice!               │
│   Carol     │ 10:02 Alice: How are you?          │
│             │────────────────────────────────────│
│             │ Type a message...                  │
└─────────────┴────────────────────────────────────┘
```

## Architecture

Seep is split into two components:

```
┌─────────────────────┐         ┌──────────────────────┐
│   seep (Rust)       │ ←─────→ │   bridge (Go)        │
│   TUI + event loop  │  Unix   │   whatsmeow          │
│   ratatui           │  socket │   WhatsApp protocol  │
└─────────────────────┘         └──────────────────────┘
```

- **Rust TUI** — handles all rendering, input, and application state
- **Go bridge** — implements the WhatsApp protocol via [whatsmeow](https://github.com/tulir/whatsmeow), communicates with the TUI over a local Unix socket

## Requirements

- Rust (1.70+)
- Go (1.21+)
- A WhatsApp account with a phone

## Installation

### 1. Clone the repo

```bash
git clone https://github.com/Guduly/seep.git
cd seep
```

### 2. Build the WhatsApp bridge

```bash
cd whatsapp
go build .
cd ..
```

### 3. Build seep

```bash
cargo build
cp whatsapp/whatsapp target/debug/whatsapp
```

### 4. Run

```bash
cargo run
```

On first run, a QR code will appear in your terminal. Scan it with WhatsApp on your phone:

1. Open WhatsApp on your phone
2. Go to **Settings → Linked Devices → Link a Device**
3. Scan the QR code

After scanning, seep will launch automatically and your session will be saved for future runs.

## Usage

### Navigation

| Key | Action |
|-----|--------|
| `←` `→` | Navigate main menu |
| `↑` `↓` | Navigate contacts |
| `Enter` | Select |
| `Esc` | Go back |
| `q` | Quit |

### Platforms

| Color | Platform |
|-------|----------|
| 🟢 Green | WhatsApp |
| 🔵 Blue | Discord |
| 🔷 Light Blue | iMessage |

### Command Line Mode

Send a message directly from the terminal without opening the TUI:

```bash
# Send a WhatsApp message
seep @alice -W "hey!"

# Send a Discord message (coming soon)
seep @alice -D "hey!"
```

## Configuration

Seep stores configuration at `~/.config/seep/config.toml`:

```toml
discord_token = "your_token_here"
```

WhatsApp sessions are stored in `whatsapp/session.db`. This file is gitignored and should never be shared.

## Security

- WhatsApp session data (`session.db`) is stored locally and never leaves your machine
- Discord tokens are stored in your OS config directory, outside the repo
- All platform communication happens over a local Unix socket (`/tmp/seep-bridge.sock`)
- No data is sent to any third-party servers

## Project Structure

```
seep/
├── src/
│   ├── main.rs       # Entry point, launches bridge
│   ├── app.rs        # Application state and logic
│   ├── event.rs      # Async event handling
│   ├── ui.rs         # All rendering (ratatui)
│   ├── config.rs     # Token persistence
│   ├── chat.rs       # Core data structures
│   └── bridge.rs     # Unix socket communication
├── whatsapp/
│   ├── main.go       # Go WhatsApp bridge
│   ├── go.mod
│   └── go.sum
└── Cargo.toml
```

## Roadmap

- [x] WhatsApp login via QR code
- [x] Load WhatsApp contacts
- [x] Send WhatsApp messages
- [ ] Receive WhatsApp messages in real time
- [ ] Scrollable contacts list
- [ ] Search contacts
- [ ] Discord support
- [ ] iMessage support
- [ ] Command line mode (`seep @name -W "message"`)
- [ ] Unread message indicators
- [ ] Message history persistence

## Disclaimer

This project uses unofficial WhatsApp APIs via whatsmeow. Use at your own risk. This is a personal project and is not affiliated with WhatsApp, Discord, or Apple.

## License

MIT
