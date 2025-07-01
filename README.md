# KeyCrafter 🔨⌨️

A terminal-based typing game disguised as a resource gathering/crafting adventure. Improve your typing speed while exploring islands and collecting resources!

## Features

- **Console-based gameplay** - Runs entirely in your terminal
- **Typing mechanics** - Type words to move your character and harvest resources
- **Pathfinding** - Your character automatically navigates to resources using A* pathfinding
- **Real-time feedback** - Letters turn green as you type them correctly, reset on mistakes
- **Resource collection** - Gather wood from trees and copper from ore deposits
- **Lightweight** - Single binary, easily distributable via SSH

## How to Play

1. **Select a resource** - Type the first letter of any word floating above a resource (🌲 tree or ⛰ copper ore)
2. **Complete the word** - Type the complete word letter by letter
   - Correct letters turn green
   - Wrong letters reset the word (you start over)
3. **Watch your character move** - Once you complete a word, your character (■) will pathfind to that resource
4. **Collect resources** - When your character reaches the resource, you'll gain materials and get a new word
5. **Quit** - Press 'q' to exit, or 'Esc' to deselect current resource

## Installation & Running

### Prerequisites
- Rust (install from https://rustup.rs/)

### Build and Run
```bash
# Clone or download the project
cd KeyCrafter

# Build and run in one command
cargo run

# Or build a release binary for distribution
cargo build --release
# Binary will be at ./target/release/keycrafter
```

### Distribution via SSH
The compiled binary is completely self-contained and can be easily shared:
```bash
# Copy to remote server
scp target/release/keycrafter user@server:/path/to/destination/

# Run on remote server
ssh user@server
./keycrafter
```

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