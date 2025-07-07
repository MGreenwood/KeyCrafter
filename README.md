# KeyCrafter 🔨⌨️

A terminal-based typing game disguised as a resource gathering/crafting adventure. Improve your typing speed while exploring islands and collecting resources!

## Quick Install

### Windows (PowerShell)
```powershell
irm play.keycrafter.fun/install.ps1 | iex
```

### Linux/Mac
```bash
curl -fsSL play.keycrafter.fun/install.sh | bash
```

After installing, just type `keycrafter` in your terminal to play!

### Manual Installation
If you prefer, you can download the binaries directly from our [GitHub releases](https://github.com/MGreenwood/KeyCrafter/releases).

## Features

- **Console-based gameplay** - Runs entirely in your terminal
- **Typing mechanics** - Type words to move your character and harvest resources
- **Pathfinding** - Your character automatically navigates to resources using A* pathfinding
- **Real-time feedback** - Letters turn green as you type them correctly, reset on mistakes
- **Resource collection** - Gather wood from trees and copper from ore deposits
- **Cross-platform** - Runs on Windows, Linux, and Mac with easy installation

## How to Play

1. **Select a resource** - Type the first letter of any word floating above a resource (🌲 tree or ⛰ copper ore)
2. **Complete the word** - Type the complete word letter by letter
   - Correct letters turn green
   - Wrong letters reset the word (you start over)
3. **Watch your character move** - Once you complete a word, your character (■) will pathfind to that resource
4. **Collect resources** - When your character reaches the resource, you'll gain materials and get a new word
5. **Quit** - Press 'q' to exit, or 'Esc' to deselect current resource

## Game Mechanics

- **Character**: Blue square (■) that moves around the island
- **Resources**: 
  - 🌲 Trees (give wood)
  - ⛰ Copper Ore (gives copper)
- **Words**: Random selection covering various keys for typing practice
- **Movement**: A* pathfinding ensures your character takes the optimal route
- **Islands**: Currently one island, with plans for multiple islands with different resources

## Controls

- **Type letters** - Select and complete words to harvest resources
- **Esc** - Deselect current resource (or quit if nothing selected)
- **q** - Quit game
- **F10** - Quick exit (works anytime)

## Save File Location

- **Installed version (from PATH):**
  - Save file is stored in your user profile directory (e.g. `%LOCALAPPDATA%\KeyCrafter\keycrafter_save.json` on Windows).
- **Running from project directory:**
  - Save file is stored in the current working directory as `keycrafter_save.json`.
- **Note:** This means your progress is separate depending on how you launch the game.

## Update & Deployment Workflow

- The update process downloads the latest binary from the server and installs it to `%LOCALAPPDATA%\KeyCrafter\keycrafter.exe`.
- The server always serves the latest build from the project directory via Docker live sync.
- To update the server binary, rebuild your project (`cargo build --release`) and the new binary will be instantly available to users.

## Troubleshooting

### Update Issues
- If `keycrafter update` does not update to the latest version:
  - Make sure you rebuilt your project and the server is serving the new binary.
  - Use the provided PowerShell script `check_keycrafter_update.ps1` to diagnose sync issues between your build, the server, and your installed version.
  - Restart your Docker container if the file does not update after a rebuild.

### Save File Issues
- If your progress is missing, check which version you are running (installed vs. project build) and look for the save file in the appropriate location.
- The save files are separate for each launch method.

## Diagnostic Script

A PowerShell script `check_keycrafter_update.ps1` is provided to:
- Show which `keycrafter.exe` is being run from your PATH
- Show details and hashes for the installed, build, and server binaries
- Show the version info from the server
- Compare all three and report if they are identical or different

## Building from Source

### Prerequisites
- Rust (install from https://rustup.rs/)

### Build Steps
```bash
# Clone the repository
git clone https://github.com/MGreenwood/KeyCrafter.git
cd KeyCrafter

# Build and run in one command
cargo build --release

# Or build a release binary
cargo build --release
```

## Technical Details

- Built with Rust for performance and easy distribution
- Uses `ratatui` for terminal UI rendering
- Uses `crossterm` for cross-platform terminal handling
- Uses `pathfinding` crate for A* pathfinding algorithm
- Lightweight dependencies, compiles to a single binary

## Future Plans

- Multiple islands with different resource types
- Crafting system using longer phrases/sentences
- Idle progression (resources accumulate while offline)
- More varied typing content (quotes, code snippets, etc.)
- Character progression and tool upgrades
- Save system for persistent progress 