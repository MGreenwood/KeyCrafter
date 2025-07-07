KeyCrafter – System & Codebase Overview

1. Current Systems
A. Typing & Movement System
Mechanics: Players type words to select and move toward resources. Each word is associated with a resource node. Typing progress is tracked per word, and only the word with a mistake is reset.
Pathfinding: Uses A* algorithm to move the player character (@) to the selected resource. Players can harvest resources within 2 moves of the target.

B. Resource & Node Management
Resource Nodes: Represented as ASCII art objects (e.g., trees, copper ore). Each node has a harvest limit (TTL), randomized per node/type.
Harvesting: When within 2 moves of a node and word is completed, the player collects resources. Nodes deplete after their TTL and may respawn elsewhere, avoiding overlap and respecting a max node count per island.

C. Word System
Words: Organized by difficulty (easy/medium/hard) with automatic space trimming.
Progression: Easy words for wood, medium for copper, with space for expansion.

D. Distribution System
Binary Distribution: Automated installation scripts and direct binary downloads.
Distribution Features:
- One-line installation commands for all platforms
- Automatic OS detection and binary selection
- PATH management for easy access
- GitHub releases for manual downloads
- Cross-platform support (Windows, Linux, Mac)
- Cloudflare-powered CDN for fast downloads

2. File Responsibilities
File	Purpose/Responsibility
main.rs	Game loop, UI rendering, input handling, main state management (Game, Player, etc.)
ascii_objects.rs	ASCII art definitions and rendering for resource nodes
coastline.rs	Coastline rendering with animated waves and land/water visualization
floating_text.rs	Floating text effect logic and management
islands.rs	Island, resource pool, and node spawning logic
pathfinding.rs	A* pathfinding implementation and grid/position logic
resource_types.rs	Shared ResourceType enum and resource-related helpers
word_lists.rs	Word management and difficulty levels
nginx/nginx.conf	Web server configuration for binary and script distribution
scripts/install.ps1	PowerShell installation script
scripts/install.sh	Bash installation script for Linux/Mac

3. Key Structs & Enums
Game (main.rs): Central game state (player, resources, islands, etc.)
Player (main.rs): Player position, inventory, and movement state
Resource (main.rs): Individual resource node state with path tracking
AsciiObject, ResourceObjects (ascii_objects.rs): ASCII art and resource node visuals
FloatingText, FloatingTextManager (floating_text.rs): Floating text effect and manager
Island, ResourcePool, IslandManager (islands.rs): Island and resource pool logic
Position, Grid (pathfinding.rs): Pathfinding and grid logic
ResourceType (resource_types.rs): Enum for all resource types
WordList, WordDifficulty (word_lists.rs): Word management and difficulty levels

4. Current Features
- Word-based movement and resource collection
- Multi-word simultaneous typing
- Distance-based harvesting (within 2 moves)
- ASCII art resource visualization
- Animated coastline with waves and clean land areas
- Floating text feedback
- Word difficulty progression
- Easy installation across platforms

5. Known Issues / Technical Debt
- State Saving: Progress is persisted between sessions (auto-save every 30 seconds)
- Single Island: Only one island currently implemented
- Planned Features Not Yet Implemented:
  - Crafting System
  - Upgrade System
  - Multiple Islands
  - Idle Progression
- No Automated Tests
- No Error Handling for Corrupt State

6. Goals & Next Steps
Short Term:
- Monitor and improve installation experience
- Improve state saving/loading (remove debug output)
- Add error handling for corrupt/missing files

Medium Term:
- Implement crafting system with sentence typing
- Add upgrade system for resource yields
- Expand to multiple islands

Long Term:
- Add idle progression
- Implement automated testing
- Enhance typing content variety
- Improve UI/UX accessibility

7. Distribution & Installation
The game is distributed through multiple channels:
- One-line installers for easy installation
- GitHub releases for manual downloads
- Cloudflare-powered CDN for fast downloads
- Automatic PATH management
- Cross-platform support

For detailed installation options, see:
- README.md: Installation instructions
- scripts/install.ps1: Windows installer
- scripts/install.sh: Linux/Mac installer
- nginx/nginx.conf: Web server configuration