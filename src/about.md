KeyCrafter – System & Codebase Overview
1. Current Systems
A. Typing & Movement System
Mechanics: Players type words to select and move toward resources. Each word is associated with a resource node. Typing progress is tracked per word, and only the word with a mistake is reset.
Pathfinding: Uses A* algorithm to move the player character (■) to the selected resource.
B. Resource & Node Management
Resource Nodes: Represented as ASCII art objects (e.g., trees, copper ore). Each node has a harvest limit (TTL), randomized per node/type.
Harvesting: On reaching a node, the player collects resources. Nodes deplete after their TTL and may respawn elsewhere, avoiding overlap and respecting a max node count per island.
C. Upgrade System
Upgrades: Players can purchase upgrades (e.g., better axe/pickaxe) to increase resource yields. Upgrades have increasing costs and are displayed in the UI with hotkeys.
D. Island & Resource Pool System
Islands: Each island has its own resource pool, spawn chance, and max node count. The system is designed for future expansion.
E. Floating Text Effects
Feedback: Floating text provides feedback for resource collection, upgrades, and node depletion. Effects float upward and fade out, rendered above the game area.
F. UI/UX
Display: Shows the game grid, resources, player, typing progress, upgrades, and help/commands.
Color Coding: Used for feedback and warnings.
Hotkeys: For quick actions and upgrades.
2. File Responsibilities
File	Purpose/Responsibility
main.rs	Game loop, UI rendering, input handling, main state management (Game, Player, etc.)
ascii_objects.rs	ASCII art definitions and rendering for resource nodes
floating_text.rs	Floating text effect logic and management
islands.rs	Island, resource pool, and node spawning logic
pathfinding.rs	A* pathfinding implementation and grid/position logic
resource_types.rs	Shared ResourceType enum and resource-related helpers
upgrades.rs	Upgrade definitions, management, and purchase logic
3. Key Structs & Enums
Game (main.rs): Central game state (player, resources, upgrades, islands, etc.)
Player (main.rs): Player position, inventory, and typing state
Resource (main.rs): Individual resource node state
AsciiObject, ResourceObjects (ascii_objects.rs): ASCII art and resource node visuals
FloatingText, FloatingTextManager (floating_text.rs): Floating text effect and manager
Island, ResourcePool, IslandManager (islands.rs): Island and resource pool logic
Position, Grid (pathfinding.rs): Pathfinding and grid logic
ResourceType (resource_types.rs): Enum for all resource types
Upgrade, UpgradeManager (upgrades.rs): Upgrade logic and state
4. Known Issues / Technical Debt
No State Saving: Progress is not persisted between sessions. (Highest priority for next feature)
Single Island: Only one island is currently implemented, though the system is designed for more.
No Crafting System: Crafting and more complex typing content are planned but not yet implemented.
Idle Progression: Not yet implemented (planned for future).
Debug Info: Some debug code and UI elements may be present in the main UI.
No Automated Tests: No unit or integration tests currently in place.
No Error Handling for Corrupt State: When state saving is implemented, error handling for corrupt/missing save files will be needed.
5. Goals & Next Steps
Implement State Saving/Loading: Serialize and persist all relevant game state (player, resources, upgrades, islands, etc.) to disk, and load on startup.
Expand Islands: Add more islands with unique resources and layouts.
Add Crafting System: Allow combining resources via typing longer phrases.
Idle Progression: Implement offline resource accumulation.
Improve Typing Content: Add more varied and challenging typing material.
Enhance UI/UX: Continue refining feedback, accessibility, and controls.
Testing & Robustness: Add tests and improve error handling, especially around save/load.
6. References
See README.md for gameplay, controls, and technical stack.
All code is modularized for easy extension and maintenance.