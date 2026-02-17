mod pathfinding;
mod ascii_objects;
mod floating_text;
mod upgrades;
mod islands;
mod resource_types;
mod crafting;
mod word_lists;
mod save_system;
mod updater;
mod coastline;

use pathfinding::{Grid, Position};
use ascii_objects::ResourceObjects;
use floating_text::FloatingTextManager;
use upgrades::UpgradeManager;
use islands::IslandManager;
use resource_types::ResourceType;
use crafting::CraftingManager;
use word_lists::{WordList, WordDifficulty};
use save_system::{SaveData, GameStats, SaveManager};
use updater::{Updater, VersionInfo};
use coastline::Coastline;
use crafting::QuestManager;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
    env,
};
use std::io::Write;

// Using Position from pathfinding module

#[derive(Clone)]
struct Resource {
    position: Position,
    resource_type: ResourceType,
    craft_sentence: String,
    next_craft_sentence: String,
    current_input: String,
    harvests_remaining: u32,
    max_harvests: u32,
    path: Vec<Position>,  // Track path for this resource
    word_start_time: Option<Instant>,  // Track timing for this specific word
}

// Using shared ResourceType from resource_types.rs

struct Player {
    position: Position,
    path: Vec<Position>,
    target: Option<Position>,
    wood: u32,
    copper: u32,
    iron: u32,
    level: u32,
    xp: u32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Self {
            position: Position::new(x, y),
            path: Vec::new(),
            target: None,
            wood: 0,
            copper: 0,
            iron: 0,
            level: 1,
            xp: 0,
        }
    }
    
    fn move_along_path(&mut self) {
        if !self.path.is_empty() {
            self.position = self.path.remove(0);
        }
    }
}

struct Game {
    player: Player,
    resources: Vec<Resource>,
    last_update: Instant,
    grid: Grid,
    resource_objects: ResourceObjects,
    floating_texts: FloatingTextManager,
    upgrades: UpgradeManager,
    island_manager: IslandManager,
    crafting: CraftingManager,
    word_list: WordList,
    save_manager: SaveManager,
    stats: GameStats,
    show_debug_info: bool,

    // Island map UI
    show_island_map: bool,
    island_map_progress: f32, // 0.0 (hidden) .. 1.0 (fully shown)
    island_map_cursor: usize, // which island is highlighted in the map

    updater: Updater,
    pending_update: Option<VersionInfo>,
    coastline: Coastline,
    quest_manager: QuestManager,

    // Crafting UI state: 0 = normal (three columns), 1 = Tools expanded, 2 = Construction expanded, 3 = Actions expanded
    crafting_expanded: u8,
    // Stats panel toggle
    show_stats: bool,
}

impl Game {
    fn new() -> Self {
        let save_manager = SaveManager::new();
        let save_data = save_manager.load_game().unwrap_or_default();
        
        let mut rng = rand::thread_rng();
        let word_list = WordList::new();
        let mut resources = Vec::new();
        let mut island_manager = IslandManager::new();
        // Restore saved current island (if valid index)
        island_manager.set_current_island(save_data.current_island_index as usize);

        // Start with half the max nodes for the restored island
        let current_island = island_manager.get_current_island();
        let initial_nodes = current_island.max_nodes / 2;
        
        // Track positions for proper spacing
        let mut existing_positions = Vec::new();
        
        // Spawn initial resources
        for _ in 0..initial_nodes {
            if let Some((x, y)) = island_manager.find_spawn_position(&existing_positions, 80, 24) {
                existing_positions.push((x, y));
                
                // Create new resource
                let resource_type = island_manager.get_random_resource_type();
                let difficulty = match resource_type {
                    ResourceType::Wood => WordDifficulty::Easy,
                    ResourceType::Copper => WordDifficulty::Medium,
                    ResourceType::Iron => WordDifficulty::Medium,
                };
                
                let (min_harvests, max_harvests) = resource_type.get_base_harvests();
                let max_harvests = rng.gen_range(min_harvests..=max_harvests);
                
                let word = word_list.get_random_word(difficulty).to_string();
                let next_word = word_list.get_random_word(difficulty).to_string();
                
                let new_resource = Resource {
                    position: Position::new(x, y),
                    resource_type,
                    craft_sentence: word,
                    next_craft_sentence: next_word,
                    current_input: String::new(),
                    harvests_remaining: max_harvests,
                    max_harvests,
                    path: Vec::new(),
                    word_start_time: None,
                };
                
                resources.push(new_resource);
            }
        }
        
        let mut grid = Grid::new();
        for resource in &resources {
            grid.add_obstacle(resource.position.clone());
        }
        
        // Start player in middle of screen
        let mut player = Player::new(40, 12);
        
        // Load saved data
        player.wood = save_data.player_wood;
        player.copper = save_data.player_copper;
        player.iron = save_data.player_iron;
        player.level = save_data.player_level;
        player.xp = save_data.player_xp;
        
        // Debug output to help track loads
        // println!("Loaded: Wood={}, Copper={}", player.wood, player.copper);
        
        // Create crafting manager and load saved state
        let mut crafting = CraftingManager::new();
        crafting.load_from_save(&save_data);
        
        // Determine initial stats-panel visibility (show by default on first run)
        let initial_show_stats = !save_manager.save_exists();
        
        let mut game = Self {
            player,
            resources,
            last_update: Instant::now(),
            grid,
            resource_objects: ResourceObjects::new(),
            floating_texts: FloatingTextManager::new(),
            upgrades: UpgradeManager::new(),
            island_manager,
            crafting,
            word_list,
            save_manager,
            stats: save_data.stats,
            show_debug_info: false,
            // Island map defaults
            show_island_map: false,
            island_map_progress: 0.0,
            island_map_cursor: 0,
            updater: Updater::new(),
            pending_update: None,
            coastline: Coastline::new(),
            quest_manager: QuestManager::new(),
            crafting_expanded: 0,
            // Stats panel visibility (toggle with '0'). Show by default on first run (no save file).
            show_stats: initial_show_stats,
        };

        // Restore saved quest completions so UI/state stays consistent
        for q in &save_data.completed_quests {
            game.quest_manager.mark_quest_completed_by_name(q);
        }

        // If the player already has a workbench (from save) but the Build-a-Workbench quest
        // is still incomplete, mark that quest complete WITHOUT granting rewards (backfill)
        if game.crafting.has_workbench && !game.quest_manager.is_quest_completed("Build a Workbench") {
            game.quest_manager.mark_quest_completed_by_name("Build a Workbench");
        }

        // ... rest of initialization ...
        game
    }
    
    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(50) {
            // Update floating texts
            self.floating_texts.update();

            // Animate island-map panel (simple linear progress)
            const MAP_STEP: f32 = 0.12; // ~8-9 frames to fully open/close
            if self.show_island_map && self.island_map_progress < 1.0 {
                self.island_map_progress = (self.island_map_progress + MAP_STEP).min(1.0);
            } else if !self.show_island_map && self.island_map_progress > 0.0 {
                self.island_map_progress = (self.island_map_progress - MAP_STEP).max(0.0);
            }

            self.last_update = now;
        }

        // Auto-save check
        if self.save_manager.should_auto_save() {
            if let Err(e) = self.save_game() {
                // eprintln!("Auto-save failed: {}", e);
            }
        }

        // Update check
        if self.updater.should_check_update() {
            if let Ok(Some(version_info)) = self.updater.check_for_updates() {
                self.pending_update = Some(version_info);
            }
        }

        self.coastline.update();
    }
    
    fn set_player_target(&mut self, target: Position) {
        // Update grid with current obstacles
        self.grid.clear_obstacles();
        
        // Add obstacles for each resource's ASCII art area, except the target
        for resource in &self.resources {
            let obj = match resource.resource_type {
                ResourceType::Wood => self.resource_objects.get("tree"),
                ResourceType::Copper => self.resource_objects.get("copper"),
                ResourceType::Iron => self.resource_objects.get("iron"),
            };
            
            if let Some(obj) = obj {
                let (w, h) = obj.dimensions();
                let rx = resource.position.x as usize;
                let ry = resource.position.y as usize;
                
                // Get the path point for this resource
                let (path_x, path_y) = obj.get_path_point(rx, ry);
                let path_pos = Position::new(path_x as i32, path_y as i32);
                
                // Add obstacles for the entire object area except the path point
                for dy in 0..h {
                    for dx in 0..w {
                        let pos = Position::new((rx + dx) as i32, (ry + dy) as i32);
                        if pos != path_pos {  // Don't block the path point
                            self.grid.add_obstacle(pos);
                        }
                    }
                }
            }
        }
        
        // Find path
        if let Some(path) = self.grid.find_path(self.player.position.clone(), target.clone()) {
            self.player.path = path;  // Keep the full path including target
            self.player.target = Some(target);
        }
    }

    /// Add XP to the player and handle level-ups. Threshold = 100 * current_level.
    fn award_xp(&mut self, mut amount: u32) {
        while amount > 0 {
            let threshold = 100u32.saturating_mul(self.player.level);
            if threshold == 0 {
                // defensive: avoid divide-by-zero
                self.player.xp = self.player.xp.saturating_add(amount);
                break;
            }

            let space = threshold.saturating_sub(self.player.xp);
            if amount < space {
                self.player.xp = self.player.xp.saturating_add(amount);
                amount = 0;
            } else {
                // fill to level, level up
                self.player.xp = 0;
                amount = amount.saturating_sub(space);
                self.player.level = self.player.level.saturating_add(1);
                self.floating_texts.add_text(
                    format!("Level up! {} → L{}", "You", self.player.level),
                    self.player.position.x as f32,
                    self.player.position.y as f32 - 2.0,
                    Color::Yellow,
                );
                // Persist on level up
                let _ = self.save_game();
            }
        }
    }
    
    fn try_spawn_resource(&mut self) {
        let current_island = self.island_manager.get_current_island();
        if (self.resources.len() as u32) < current_island.max_nodes {
            if self.island_manager.should_spawn_node() {
                self.spawn_new_resource();
            }
        }
    }

    fn try_spawn_resource_on_word_completion(&mut self) {
        let current_island = self.island_manager.get_current_island();
        if (self.resources.len() as u32) < current_island.max_nodes {
            // Higher chance to spawn on word completion (50% chance vs normal spawn rate)
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.5) {
                self.spawn_new_resource();
            }
        }
    }

    fn spawn_new_resource(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Get existing positions
        let existing_positions: Vec<(i32, i32)> = self.resources
            .iter()
            .map(|r| (r.position.x, r.position.y))
            .collect();

        // Try to find a spawn position
        if let Some((x, y)) = self.island_manager.find_spawn_position(&existing_positions, 80, 24) {
            // Create new resource
            let resource_type = self.island_manager.get_random_resource_type();
            let difficulty = match resource_type {
                ResourceType::Wood => WordDifficulty::Easy,
                ResourceType::Copper => WordDifficulty::Medium,
                ResourceType::Iron => WordDifficulty::Medium,
            };
            
            let (min_harvests, max_harvests) = resource_type.get_base_harvests();
            let max_harvests = rng.gen_range(min_harvests..=max_harvests);
            
            let word = self.word_list.get_random_word(difficulty).to_string();
            let next_word = self.word_list.get_random_word(difficulty).to_string();
            
            let new_resource = Resource {
                position: Position::new(x, y),
                resource_type,
                craft_sentence: word,
                next_craft_sentence: next_word,
                current_input: String::new(),
                harvests_remaining: max_harvests,
                max_harvests,
                path: Vec::new(),
                word_start_time: None,
            };
            
            // Add the resource and update the grid
            self.grid.add_obstacle(new_resource.position.clone());
            self.resources.push(new_resource);
            
            // Show spawn notification
            self.floating_texts.add_text(
                "New Resource!".to_string(),
                x as f32,
                y as f32 - 1.0,
                Color::Cyan
            );
        }
    }

    fn harvest_resource(&mut self) {
        // First find the index of the resource to harvest
        let mut harvest_idx = None;
        let mut harvest_amount = 0;
        let mut harvest_text = String::new();
        let mut harvest_color = Color::White;

        // Find the resource to harvest
        for (idx, resource) in self.resources.iter().enumerate() {
            let target_pos = if let Some(obj) = match resource.resource_type {
                ResourceType::Wood => self.resource_objects.get("tree"),
                ResourceType::Copper => self.resource_objects.get("copper"),
                ResourceType::Iron => self.resource_objects.get("iron"),
            } {
                let (x, y) = obj.get_path_point(resource.position.x as usize, resource.position.y as usize);
                Position::new(x as i32, y as i32)
            } else {
                resource.position.clone()
            }; 

            let distance = self.player.position.manhattan_distance(&target_pos);
            if distance <= 2 && resource.current_input == resource.craft_sentence {
                // Calculate harvest amount and text
                let (amount, text, color) = match resource.resource_type {
                    ResourceType::Wood => {
                        let multiplier = self.crafting.get_multiplier(&ResourceType::Wood);
                        let amount = (multiplier as u32).max(1);
                        self.player.wood += amount;
                        self.stats.add_resource_harvested(ResourceType::Wood, amount);
                        (amount, "Wood".to_string(), ResourceType::Wood.get_color())
                    },
                    ResourceType::Copper => {
                        let multiplier = self.crafting.get_multiplier(&ResourceType::Copper);
                        let amount = (multiplier as u32).max(1);
                        self.player.copper += amount;
                        self.stats.add_resource_harvested(ResourceType::Copper, amount);
                        (amount, "Copper".to_string(), ResourceType::Copper.get_color())
                    },
                    ResourceType::Iron => {
                        let multiplier = self.crafting.get_multiplier(&ResourceType::Iron);
                        let amount = (multiplier as u32).max(1);
                        let prev_iron = self.player.iron;
                        self.player.iron += amount;
                        self.stats.add_resource_harvested(ResourceType::Iron, amount);

                        // Unlock Iron Sword the first time the player collects any iron
                        if prev_iron == 0 {
                            if self.crafting.unlock_recipe("Iron Sword") {
                                // show a notification and persist the unlock
                                self.floating_texts.add_text(
                                    "New craft unlocked: Iron Sword".to_string(),
                                    self.player.position.x as f32,
                                    self.player.position.y as f32 - 1.0,
                                    Color::Yellow,
                                );
                                let _ = self.save_game();
                            }
                        }

                        (amount, "Iron".to_string(), ResourceType::Iron.get_color())
                    },

                };
                harvest_idx = Some(idx);
                harvest_amount = amount;
                harvest_text = text;
                harvest_color = color;
                break;
            }
        }

        // Process the harvest if we found a resource
        if let Some(idx) = harvest_idx {
            // Show floating text
            self.floating_texts.add_text(
                format!("+{} {}", harvest_amount, harvest_text),
                self.player.position.x as f32,
                self.player.position.y as f32 - 1.0,
                harvest_color
            );

            // Update the resource
            if let Some(resource) = self.resources.get_mut(idx) {
                resource.harvests_remaining = resource.harvests_remaining.saturating_sub(1);
                resource.current_input.clear();

                // Check if this was the last node and it's depleted
                if resource.harvests_remaining == 0 {
                    // Remove depleted resources
                    self.resources.retain(|r| r.harvests_remaining > 0);

                    // If no resources left, respawn max_nodes
                    if self.resources.is_empty() {
                        let current_island = self.island_manager.get_current_island();
                        self.floating_texts.add_text(
                            "CLEAR! Respawning nodes...".to_string(),
                            40.0, // Center of screen
                            12.0,
                            Color::Cyan
                        );

                        // Spawn max_nodes new resources
                        let mut rng = rand::thread_rng();
                        let mut existing_positions = Vec::new();
                        for _ in 0..current_island.max_nodes {
                            if let Some((x, y)) = self.island_manager.find_spawn_position(&existing_positions, 80, 24) {
                                existing_positions.push((x, y));
                                
                                // Create new resource
                                let resource_type = self.island_manager.get_random_resource_type();
                                let difficulty = match resource_type {
                                    ResourceType::Wood => WordDifficulty::Easy,
                                    ResourceType::Copper => WordDifficulty::Medium,
                                    ResourceType::Iron => WordDifficulty::Medium,
                                };
                                
                                let (min_harvests, max_harvests) = resource_type.get_base_harvests();
                                let max_harvests = rng.gen_range(min_harvests..=max_harvests);
                                
                                let word = self.word_list.get_random_word(difficulty).to_string();
                                let next_word = self.word_list.get_random_word(difficulty).to_string();
                                
                                let new_resource = Resource {
                                    position: Position::new(x, y),
                                    resource_type,
                                    craft_sentence: word,
                                    next_craft_sentence: next_word,
                                    current_input: String::new(),
                                    harvests_remaining: max_harvests,
                                    max_harvests,
                                    path: Vec::new(),
                                    word_start_time: None,
                                };
                                
                                self.resources.push(new_resource);
                            }
                        }
                    }
                }
            }

            // Try to spawn a new resource
            self.try_spawn_resource();
        }
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> Option<VersionInfo> {
        // Stop showing debug info after first key press
        self.show_debug_info = false;

        // If the island map is open, handle map-specific keys (Tab to cycle, Space/Enter to select, m to close)
        if self.island_map_progress > 0.0 {
            match key.code {
                KeyCode::Tab => {
                    let count = self.island_manager.island_count();
                    if count > 0 {
                        self.island_map_cursor = (self.island_map_cursor + 1) % count;
                    }
                    return None;
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    let idx = self.island_map_cursor;

                    // Already on this island?
                    if idx == self.island_manager.get_current_island_index() {
                        self.floating_texts.add_text(
                            "You're already on this island.".to_string(),
                            40.0,
                            6.0,
                            Color::Gray,
                        );
                        return None;
                    }

                    // Level requirement check
                    if let Some(req) = self.island_manager.get_island_level_requirement(idx) {
                        if self.player.level < req {
                            let name = self.island_manager.get_island_name(idx).unwrap_or_else(|| "Unknown".to_string());
                            self.floating_texts.add_text(
                                format!("Level {} required to travel to {}!", req, name),
                                40.0,
                                6.0,
                                Color::Red,
                            );
                            return None;
                        }
                    }

                    // Need a boat to travel
                    let has_boat = self.crafting.get_completed_items().iter().any(|it| it == "Boat");
                    if !has_boat {
                        self.floating_texts.add_text(
                            "You need a Boat to travel!".to_string(),
                            40.0,
                            6.0,
                            Color::Red,
                        );
                        return None;
                    }

                    // Perform travel: switch island and respawn nodes
                    self.island_manager.set_current_island(idx);
                    let current_island = self.island_manager.get_current_island();

                    // Clear and spawn new resources for the selected island
                    self.resources.clear();
                    self.grid.clear_obstacles();
                    let mut existing_positions: Vec<(i32, i32)> = Vec::new();
                    let mut rng = rand::thread_rng();
                    for _ in 0..current_island.max_nodes {
                        if let Some((x, y)) = self.island_manager.find_spawn_position(&existing_positions, 80, 24) {
                            existing_positions.push((x, y));
                            let resource_type = self.island_manager.get_random_resource_type();
                            let difficulty = match resource_type {
                                ResourceType::Wood => WordDifficulty::Easy,
                                ResourceType::Copper => WordDifficulty::Medium,
                                ResourceType::Iron => WordDifficulty::Medium,
                            };
                            let (min_harvests, max_harvests) = resource_type.get_base_harvests();
                            let max_harvests = rng.gen_range(min_harvests..=max_harvests);
                            let word = self.word_list.get_random_word(difficulty).to_string();
                            let next_word = self.word_list.get_random_word(difficulty).to_string();
                            let new_resource = Resource {
                                position: Position::new(x, y),
                                resource_type,
                                craft_sentence: word,
                                next_craft_sentence: next_word,
                                current_input: String::new(),
                                harvests_remaining: max_harvests,
                                max_harvests,
                                path: Vec::new(),
                                word_start_time: None,
                            };
                            self.grid.add_obstacle(new_resource.position.clone());
                            self.resources.push(new_resource);
                        }
                    }

                    // Center player and show confirmation
                    self.player.position = Position::new(40, 12);
                    self.floating_texts.add_text(
                        format!("Sailed to {}!", current_island.name),
                        40.0,
                        6.0,
                        Color::Cyan,
                    );

                    // Persist the player's current island immediately
                    let _ = self.save_game();

                    // Close the map
                    self.show_island_map = false;
                    return None;
                }
                KeyCode::Char('m') => {
                    self.show_island_map = false;
                    return None;
                }
                _ => return None,
            }
        }

        match key.code {
            KeyCode::Char('u') if self.pending_update.is_some() => {
                // Clone version info before any mutable borrow
                let version_info = self.pending_update.as_ref().cloned();
                if let Some(version_info) = version_info {
                    let _ = self.save_game();
                    match self.updater.download_update(&version_info) {
                        Ok(new_exe_path) => {
                            if let Err(e) = self.updater.apply_update(&new_exe_path) {
                                self.floating_texts.add_text(format!("Update failed: {}", e), 40.0, 10.0, Color::Red);
                            } else {
                                // Signal that we want to exit for update
                                return Some(version_info);
                            }
                        }
                        Err(e) => self.floating_texts.add_text(format!("Update download failed: {}", e), 40.0, 10.0, Color::Red),
                    }
                }
            }

            // Crafting area column hotkeys: 1 = Tools, 2 = Construction, 3 = Actions
            KeyCode::Char('1') => {
                self.crafting_expanded = if self.crafting_expanded == 1 { 0 } else { 1 };
                return None;
            }
            KeyCode::Char('2') => {
                self.crafting_expanded = if self.crafting_expanded == 2 { 0 } else { 2 };
                return None;
            }
            KeyCode::Char('3') => {
                self.crafting_expanded = if self.crafting_expanded == 3 { 0 } else { 3 };
                return None;
            }

            // Toggle stats panel (bottom-right) with '0'
            KeyCode::Char('0') => {
                self.show_stats = !self.show_stats;
                return None;
            }

            KeyCode::Char(c) => {
                // Handle crafting input - check all recipes simultaneously
                let mut crafting_completed = false;
                let mut completed_recipe_idx = None;
                let mut any_crafting_progress = false;
                
                for recipe_idx in 0..self.crafting.get_recipes().len() {
                    // Allow typing the craft sentence for any unlocked recipe (even if resources are missing)
                    if self.crafting.is_recipe_unlocked(recipe_idx) {
                        if self.crafting.handle_input(recipe_idx, c) {
                            any_crafting_progress = true;

                            // If the sentence is fully typed, attempt to craft only when resources are available
                            let recipe_ref = &self.crafting.get_recipes()[recipe_idx];
                            if recipe_ref.current_input == recipe_ref.craft_sentence {
                                if self.crafting.can_craft(recipe_idx, self.player.wood, self.player.copper) {
                                    if let Some((recipe, costs)) = self.crafting.craft_item(recipe_idx) {
                                        self.stats.add_successful_craft();

                                        // Deduct resources
                                        for (resource_type, amount) in costs {
                                            match resource_type {
                                                ResourceType::Wood => self.player.wood -= amount,
                                                ResourceType::Copper => self.player.copper -= amount,
                                                ResourceType::Iron => self.player.iron -= amount,
                                            }
                                        }

                                        // Show crafting success message
                                        self.floating_texts.add_text(
                                            format!("Crafted {}!", recipe.name),
                                            self.player.position.x as f32,
                                            self.player.position.y as f32 - 1.0,
                                            Color::Yellow
                                        );

                                        // If this was the workbench, show unlock message
                                        if recipe_idx == 0 {
                                            self.floating_texts.add_text(
                                                "New recipes unlocked!".to_string(),
                                                self.player.position.x as f32,
                                                self.player.position.y as f32 - 2.0,
                                                Color::Cyan
                                            );

                                                                    // Check and complete the "Build a Workbench" quest (grant rewards)
                                    if let Some(quest) = self.quest_manager.get_current_quest() {
                                        let quest_title = quest.title.clone();
                                        if quest_title == "Build a Workbench" {
                                            let rewards = quest.rewards.clone();
                                            // Mark quest complete
                                            self.quest_manager.complete_current_quest();

                                                    // Grant rewards to player and show floating text
                                                    for (res, amt) in &rewards {
                                                        match res {
                                                            ResourceType::Wood => self.player.wood += *amt,
                                                            ResourceType::Copper => self.player.copper += *amt,
                                                            ResourceType::Iron => self.player.iron += *amt,
                                                        }
                                                        self.floating_texts.add_text(
                                                            format!("+{} {}", amt, res.get_display_name()),
                                                            self.player.position.x as f32,
                                                            self.player.position.y as f32 - 3.0,
                                                            res.get_color()
                                                        );
                                                    }

                                                    // Award XP for quest completion (flat)
                                                    self.award_xp(50);
                                                }
                                            }
                                        }

                                        // If player just crafted the Sail action, open the island map (typing required)
                                        if recipe.name == "Sail" {
                                            // Open map immediately — player must type the sail phrase to trigger this
                                            self.show_island_map = true;
                                            self.island_map_progress = 0.0;
                                            self.island_map_cursor = self.island_manager.get_current_island_index();

                                            self.floating_texts.add_text(
                                                "You set the sails — island map opened!".to_string(),
                                                self.player.position.x as f32,
                                                self.player.position.y as f32 - 2.0,
                                                Color::Cyan
                                            );
                                        }
                                    }
                                } else {
                                    // Full sentence typed but missing resources — notify player
                                    self.floating_texts.add_text(
                                        "Not enough resources to craft.".to_string(),
                                        self.player.position.x as f32,
                                        self.player.position.y as f32 - 1.0,
                                        Color::Red,
                                    );
                                    // Clear the recipe's typed input so the UI doesn't stay highlighted as 'complete'
                                    self.crafting.clear_input(recipe_idx);
                                }
                            }
                        }
                    }
                }

                // If crafting was completed, clear other recipe inputs and don't process resource gathering
                if crafting_completed {
                    // Clear inputs for other recipes to prevent conflicts
                    for recipe_idx in 0..self.crafting.get_recipes().len() {
                        if Some(recipe_idx) != completed_recipe_idx {
                            self.crafting.clear_input(recipe_idx);
                        }
                    }
                    return None;
                }

                // If any crafting input was in progress, don't process resource gathering
                if any_crafting_progress {
                    return None;
                }

                // If not crafting, handle resource gathering input
                let mut should_harvest = false;
                let mut completed_word_idx = None;
                let mut word_completed = false;
                // Accumulate XP to award after resource loop (avoid mutable-borrow conflicts)
                let mut xp_to_award: u32 = 0;

                // First collect all resource positions and their obstacles
                let mut resource_obstacles = Vec::new();
                for resource in &self.resources {
                    let obj = match resource.resource_type {
                        ResourceType::Wood => self.resource_objects.get("tree"),
                        ResourceType::Copper => self.resource_objects.get("copper"),
                        ResourceType::Iron => self.resource_objects.get("iron"),
                    };
                    
                    if let Some(obj) = obj {
                        let (w, h) = obj.dimensions();
                        let rx = resource.position.x as usize;
                        let ry = resource.position.y as usize;
                        resource_obstacles.push((resource.position.clone(), (rx, ry, w, h)));
                    }
                }

                // Process each word independently
                for (resource_idx, resource) in self.resources.iter_mut().enumerate() {
                    let current_pos = resource.current_input.len();
                    let target_word = &resource.craft_sentence;

                    // If we haven't started this word yet, check if this is the first letter
                    if current_pos == 0 {
                        let expected = target_word.chars().next();
                        if expected == Some(c) {
                            // Start this word
                            resource.current_input.push(c);
                            resource.word_start_time = Some(Instant::now());
                            
                            // Calculate initial path
                            let target_pos = if let Some(obj) = match resource.resource_type {
                                ResourceType::Wood => self.resource_objects.get("tree"),
                                ResourceType::Copper => self.resource_objects.get("copper"),
                                ResourceType::Iron => self.resource_objects.get("iron"),
                            } {
                                let (x, y) = obj.get_path_point(resource.position.x as usize, resource.position.y as usize);
                                Position::new(x as i32, y as i32)
                            } else {
                                resource.position.clone()
                            };
                            
                            // Clear and rebuild grid obstacles
                            self.grid.clear_obstacles();
                            for (pos, (rx, ry, w, h)) in &resource_obstacles {
                                if *pos != resource.position {  // Don't block target
                                    // Add obstacles for the object area
                                    for dy in 0..*h {
                                        for dx in 0..*w {
                                            let obstacle_pos = Position::new((*rx + dx) as i32, (*ry + dy) as i32);
                                            if obstacle_pos != target_pos {  // Don't block the actual target point
                                                self.grid.add_obstacle(obstacle_pos);
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(path) = self.grid.find_path(self.player.position.clone(), target_pos.clone()) {
                                resource.path = path;  // Store path in the resource
                                self.player.target = Some(target_pos);
                            }

                            // Move first step
                            if !resource.path.is_empty() {
                                self.player.position = resource.path.remove(0);
                            }
                        }
                    }
                    // If we've started this word, continue it
                    else if !resource.current_input.is_empty() {
                        let expected = target_word.chars().nth(current_pos);
                        if expected == Some(c) {
                            // Continue the word
                            resource.current_input.push(c);

                            // Move one step
                            if !resource.path.is_empty() {
                                self.player.position = resource.path.remove(0);
                            }

                            // Check if word is complete
                            if resource.current_input == *target_word {
                                completed_word_idx = Some(resource_idx);
                                word_completed = true;
                                
                                // Track word completion stats
                                if let Some(start_time) = resource.word_start_time {
                                    let time_taken = start_time.elapsed().as_secs_f32();
                                    self.stats.add_word_completed(target_word.len() as u32, time_taken);

                                    // Award XP per letter for typing words correctly (queue to avoid borrow conflicts)
                                    let xp_gain = target_word.chars().filter(|c| !c.is_whitespace()).count() as u32;
                                    xp_to_award = xp_to_award.saturating_add(xp_gain);
                                }
                                resource.word_start_time = None;
                                
                                // Get the target position
                                let target_pos = if let Some(obj) = match resource.resource_type {
                                    ResourceType::Wood => self.resource_objects.get("tree"),
                                    ResourceType::Copper => self.resource_objects.get("copper"),
                                    ResourceType::Iron => self.resource_objects.get("iron"),
                                } {
                                    let (x, y) = obj.get_path_point(resource.position.x as usize, resource.position.y as usize);
                                    Position::new(x as i32, y as i32)
                                } else {
                                    resource.position.clone()
                                };

                                let distance = self.player.position.manhattan_distance(&target_pos);
                                if distance <= 2 {
                                    should_harvest = true;
                                } 
                            }
                        } else {
                            // Wrong letter, clear this word
                            self.stats.add_mistake();
                            resource.word_start_time = None;
                            resource.current_input.clear();
                            resource.path.clear();
                        }
                    }
                }

                // Handle harvest after the loop
                if should_harvest {
                    self.harvest_resource();
                    self.player.target = None;
                }

                // Replace completed word with a new one
                if let Some(idx) = completed_word_idx {
                    if idx < self.resources.len() {
                        self.replace_word(idx);
                    }
                }

                // Try to spawn a new resource when word is completed
                if word_completed {
                    self.try_spawn_resource_on_word_completion();
                }

                // Award queued XP (outside of the mutable resources borrow)
                if xp_to_award > 0 {
                    self.award_xp(xp_to_award);
                }
            }
            _ => {} // Ignore other key events
        }
        None
    }

    fn render_game_area(&self, f: &mut Frame, game_area: Rect) {
        let mut lines = Vec::new();
        
        // Create empty grid
        for y in 0..game_area.height {
            let mut line_spans = Vec::new();
            for x in 0..game_area.width {
                let pos = Position::new(x as i32, y as i32);
                
                // Add resource counter at top-right if we're at the right position
                if y == 0 && x >= game_area.width.saturating_sub(40) {
                    if x == game_area.width.saturating_sub(40) {
                        let wood_text = format!("Wood: {}", self.player.wood);
                        let copper_text = format!("Copper: {}", self.player.copper);
                        let iron_text = format!("Iron: {}", self.player.iron);
                        line_spans.push(Span::styled(
                            wood_text,
                            Style::default().fg(ResourceType::Wood.get_color())
                        ));
                        line_spans.push(Span::raw(" | "));
                        line_spans.push(Span::styled(
                            copper_text,
                            Style::default().fg(ResourceType::Copper.get_color())
                        ));
                        line_spans.push(Span::raw(" | "));
                        line_spans.push(Span::styled(
                            iron_text,
                            Style::default().fg(ResourceType::Iron.get_color())
                        ));
                        // Small hint: toggle player stats panel with '0' (brighter/bold to improve discoverability)
                        line_spans.push(Span::raw("  "));
                        line_spans.push(Span::styled(
                            "[0] Stats",
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        ));
                        // Skip the rest of this line
                        break;
                    }
                    continue;
                }
                
                // Reserve right column for overlays (quest / completed items will be rendered as a Paragraph)
                const RIGHT_COL_WIDTH: u16 = 40;
                if x >= game_area.width.saturating_sub(RIGHT_COL_WIDTH) {
                    // Leave space in the grid here; we'll draw the full right-column UI as a Paragraph later
                    line_spans.push(Span::raw(" "));
                    continue;
                }
                
                // Get the coastline/water tile first
                let (coast_char, coast_style) = self.coastline.get_tile(
                    x as i32, 
                    y as i32, 
                    game_area.width as i32,
                    game_area.height as i32
                );
                
                // Check if player is here
                let span = if pos == self.player.position {
                    Span::styled("@", Style::default().fg(Color::Blue))
                } else {
                    // Check if this position is part of any resource's ASCII art
                    let mut found_char = None;
                    for resource in &self.resources {
                        let obj = match resource.resource_type {
                            ResourceType::Wood => self.resource_objects.get("tree"),
                            ResourceType::Copper => self.resource_objects.get("copper"),
                            ResourceType::Iron => self.resource_objects.get("iron"),
                        };
                        
                        if let Some(obj) = obj {
                            let rx = resource.position.x as usize;
                            let ry = resource.position.y as usize;
                            let chars = obj.render_at(rx, ry);
                            if let Some((_, _, c)) = chars.iter().find(|(x, y, _)| *x == pos.x as usize && *y == pos.y as usize) {
                                found_char = Some((*c, Style::default().fg(resource.resource_type.get_color())));
                                break;
                            }
                        }
                    }
                    
                    if let Some((c, style)) = found_char {
                        Span::styled(c.to_string(), style)
                    } else {
                        // Check if we need to render a word above a resource
                        let mut word_span = None;
                        for resource in &self.resources {
                            let rx = resource.position.x as usize;
                            let ry = resource.position.y as usize;
                            
                            // Position the word centered above the resource
                            if y as usize == ry - 1 {
                                let word_start = rx.saturating_sub(resource.craft_sentence.len() / 2);
                                let word_end = word_start + resource.craft_sentence.len();
                                let x_pos = x as usize;
                                
                                // Current word
                                if x_pos >= word_start && x_pos < word_end {
                                    let char_idx = x_pos - word_start;
                                    if let Some(c) = resource.craft_sentence.chars().nth(char_idx) {
                                        let style = if char_idx < resource.current_input.len() {
                                            Style::default().fg(Color::Green)
                                        } else {
                                            Style::default().fg(Color::White)
                                        };
                                        word_span = Some(Span::styled(c.to_string(), style));
                                    }
                                }
                                // Next word (if not on last harvest)
                                else if resource.harvests_remaining > 1 {
                                    let next_start = word_end + 1; // One space after current word
                                    let next_end = next_start + resource.next_craft_sentence.len();
                                    if x_pos >= next_start && x_pos < next_end {
                                        let char_idx = x_pos - next_start;
                                        if let Some(c) = resource.next_craft_sentence.chars().nth(char_idx) {
                                            word_span = Some(Span::styled(
                                                c.to_string(),
                                                Style::default().fg(Color::DarkGray)
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        
                        word_span.unwrap_or_else(|| Span::styled(coast_char, coast_style))
                    }
                };
                line_spans.push(span);
            }
            lines.push(Line::from(line_spans));
        }
        
        // First render the game background and objects
        let game_widget = Paragraph::new(lines.clone())
            .block(Block::default().borders(Borders::ALL).title(format!("KeyCrafter - {}", self.island_manager.get_current_island().name)));
        f.render_widget(game_widget, game_area);

        // Then render floating texts on top
        for floating_text in self.floating_texts.get_texts() {
            let (x, y) = floating_text.get_position();
            // Only render floating text within the game area bounds (excluding borders)
            if y > 0 && y < (game_area.height - 1) as usize && 
               x > 0 && x < (game_area.width - 1) as usize {
                
                // Adjust for the border offset
                let adjusted_y = y - 1;
                let adjusted_x = x - 1;
                
                if adjusted_y < lines.len() && adjusted_x < game_area.width as usize {
                    // Get what's currently at this position
                    let current_line = &lines[adjusted_y];
                    let mut new_line = current_line.spans.clone();
                    
                    // Calculate where in the line to insert the text
                    let text = floating_text.get_text();
                    let start_x = adjusted_x.min((game_area.width - 2) as usize - text.len());
                    
                    // Replace spans at the text position, but only within bounds
                    for (i, c) in text.chars().enumerate() {
                        let pos_x = start_x + i;
                        if pos_x < new_line.len() && pos_x < (game_area.width - 2) as usize {
                            let color = floating_text.get_color();
                            new_line[pos_x] = Span::styled(
                                c.to_string(),
                                Style::default().fg(color).add_modifier(Modifier::BOLD)
                            );
                        }
                    }
                    
                    // Render just this line within the game area
                    let text_pos = Rect::new(
                        game_area.x + 1, // Account for border
                        game_area.y + 1 + adjusted_y as u16, // Account for border and line position
                        game_area.width - 2, // Account for borders
                        1,
                    );
                    let text_widget = Paragraph::new(Line::from(new_line));
                    f.render_widget(text_widget, text_pos);
                }
            }
        }

        // Render right-column quest + completed items as a Paragraph to avoid per-character spacing
        const RIGHT_COL_WIDTH: u16 = 40;
        const RIGHT_COL_TOP_OFFSET: u16 = 4; // move quests down so top-right resource counter remains visible
        if game_area.width > RIGHT_COL_WIDTH + 4 {
            let right_rect = Rect::new(
                game_area.x + game_area.width - RIGHT_COL_WIDTH - 1, // inside the border
                game_area.y + RIGHT_COL_TOP_OFFSET,
                RIGHT_COL_WIDTH,
                game_area.height.saturating_sub(RIGHT_COL_TOP_OFFSET + 1),
            );

            // Build lines for quest + completed items
            let mut right_lines: Vec<Line> = Vec::new();
            if let Some(quest) = self.quest_manager.get_current_quest() {
                // Rewards line
                let mut reward_spans = Vec::new();
                for (res, amt) in &quest.rewards {
                    reward_spans.push(Span::styled(
                        format!("+{} {}  ", amt, res.get_display_name()),
                        Style::default().fg(res.get_color()).add_modifier(Modifier::BOLD)
                    ));
                }
                right_lines.push(Line::from(reward_spans));

                // Title and description (wrapped by Paragraph)
                right_lines.push(Line::from(vec![Span::styled(&quest.title, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]));
                right_lines.push(Line::from(vec![Span::styled(&quest.description, Style::default().fg(Color::Gray))]));
                right_lines.push(Line::from(Span::raw("")));
            }

            // Completed items
            for item in self.crafting.get_completed_items() {
                right_lines.push(Line::from(vec![Span::styled(item, Style::default().fg(Color::Green))]));
            }

            let right_widget = Paragraph::new(right_lines)
                .block(Block::default())
                .wrap(Wrap { trim: true });

            f.render_widget(Clear, right_rect);
            f.render_widget(right_widget, right_rect);


        }

        // Show update notification if available
        if let Some(version_info) = &self.pending_update {
            let message = self.updater.get_update_message(version_info);
            let lines: Vec<Line> = message.lines().map(|line| {
                Line::from(vec![
                    Span::styled(line, Style::default().fg(Color::Yellow))
                ])
            }).collect();

            let update_area = Rect::new(
                game_area.x + (game_area.width / 4),
                game_area.y + (game_area.height / 4),
                game_area.width / 2,
                (lines.len() + 2) as u16,
            );

            let update_widget = Paragraph::new(lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Update Available"))
                .alignment(Alignment::Center);

            f.render_widget(Clear, update_area);
            f.render_widget(update_widget, update_area);
        }

        // Island map overlay (animated slide-up)
        self.render_island_map(f, game_area);

        // Stats panel (toggle with '0') - bottom-right of play area
        if self.show_stats {
            const STATS_W: u16 = 30;
            // Increase height so the Level/XP line fits (content lines = STATS_H - 2)
            const STATS_H: u16 = 6;
            const RIGHT_COL_WIDTH: u16 = 40;
            let mut sx = game_area.x + game_area.width.saturating_sub(STATS_W + 2);
            if game_area.width > RIGHT_COL_WIDTH + 8 {
                sx = game_area.x + game_area.width.saturating_sub(RIGHT_COL_WIDTH + STATS_W + 4);
            }
            let sy = game_area.y + game_area.height.saturating_sub(STATS_H + 1);

            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(vec![Span::styled("Player Stats [0]", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]));
            lines.push(Line::from(vec![Span::raw(format!("Words: {}  WPM: {:.1}", self.stats.words_typed, self.stats.average_wpm))]));
            let acc = self.stats.get_accuracy_percentage();
            lines.push(Line::from(vec![Span::raw(format!("Mistakes: {}  Acc: {:.0}%", self.stats.mistakes_made, acc))]));
            let lvl = self.player.level;
            let xp = self.player.xp;
            let xp_next = 100u32.saturating_mul(lvl);
            let pct = if xp_next > 0 { (xp as f32 / xp_next as f32) * 100.0 } else { 0.0 };
            lines.push(Line::from(vec![Span::raw(format!("Level: {}  XP: {}/{} ({:.0}%)", lvl, xp, xp_next, pct))]));

            let stats_widget = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Stats"))
                .wrap(Wrap { trim: true });
            f.render_widget(stats_widget, Rect::new(sx, sy, STATS_W, STATS_H));
        }

        // Show debug info at the bottom if enabled
        if self.show_debug_info {
            let debug_text = format!("Loaded: Wood={}, Copper={}, Iron={}", self.player.wood, self.player.copper, self.player.iron);
            let debug_pos = Rect::new(
                game_area.x + 1,
                game_area.y + game_area.height - 2,
                game_area.width - 2,
                1,
            );
            let debug_widget = Paragraph::new(Line::from(vec![
                Span::styled(debug_text, Style::default().fg(Color::Gray))
            ]));
            f.render_widget(debug_widget, debug_pos);
        }
    }

    // Animated island-map pop-up rendered from the bottom of the game area
    fn render_island_map(&self, f: &mut Frame, game_area: Rect) {
        // Only draw when at least partly visible
        if self.island_map_progress <= 0.01 {
            return;
        }

        // Dimensions
        let max_h: u16 = 9; // expanded panel when fully shown
        let h = ((max_h as f32) * self.island_map_progress).max(3.0).round() as u16;
        let w = (game_area.width.saturating_sub(8)).max(40);
        let x = game_area.x + ((game_area.width.saturating_sub(w)) / 2);
        let y = game_area.y + game_area.height.saturating_sub(h + 1);
        let area = Rect::new(x, y, w, h);

        // Wave/ship animation phase (driven by system time)
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        let phase = (millis / 200) as usize;
        let wave_chars = ['~', '-', '`', '.'];

        // Build map lines
        let mut lines: Vec<Line> = Vec::new();

        // Top decorative waving line
        let mut top_wave = String::new();
        for i in 0..(w as usize) {
            let ch = wave_chars[(i + phase) % wave_chars.len()];
            top_wave.push(ch);
        }
        lines.push(Line::from(vec![Span::styled(top_wave, Style::default().fg(Color::Blue))]));

        // Spacer
        lines.push(Line::from(Span::raw(" ")));

        // Island ASCII rows (3 rows) + labels
        let left_art = [" /\\ ", "/~~\\", " || "];
        let right_art = [" /\\ ", "(Fe)", "\\__/"];

        // Precompute names and lengths so labels only appear on the middle row
        let mut left_name = self.island_manager.get_island_name(0).unwrap_or_else(|| "Tutorial Island".to_string());
        let mut right_name = self.island_manager.get_island_name(1).unwrap_or_else(|| "Iron Mountains".to_string());
        let right_level = self.island_manager.get_island_level_requirement(1).unwrap_or(5);
        let mut right_label = format!("{} (Lvl {})", right_name, right_level);

        // Truncate labels if the panel is narrow
        let max_label = (w as usize).saturating_sub(24).max(8);
        if left_name.len() > max_label { left_name = format!("{}...", &left_name[..max_label.saturating_sub(3)]); }
        if right_label.len() > max_label { right_label = format!("{}...", &right_label[..max_label.saturating_sub(3)]); }

        let left_label_len = left_name.len();
        let right_label_len = right_label.len();
        let left_art_w = 4usize; // " /\\ " width
        let right_art_w = 4usize; // " /\\ " or "(Fe)" width

        for row in 0..3 {
            let mut spans = Vec::new();

            // Styles (highlight selected)
            let left_style = if self.island_map_cursor == 0 {
                Style::default().fg(ResourceType::Wood.get_color()).add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default().fg(ResourceType::Wood.get_color())
            };
            let right_style = if self.island_map_cursor == 1 {
                Style::default().fg(ResourceType::Iron.get_color()).add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default().fg(ResourceType::Iron.get_color())
            };

            // Build line by placing pieces at explicit columns so wrapping/duplication can't occur
            // Left art
            spans.push(Span::styled(left_art[row], left_style));
            spans.push(Span::raw("  "));

            // Left label only on middle row
            if row == 1 {
                spans.push(Span::styled(left_name.clone(), left_style));
            } else {
                spans.push(Span::raw(" ".repeat(left_label_len)));
            }

            // compute where to place the right island so it fits within width
            let current_len = left_art_w + 2 + left_label_len; // characters so far
            // shift right-island left by N columns (keeps it from hugging the far-right)
            let desired_left_shift = 15usize;
            let right_edge_start = if (right_art_w + 2 + right_label_len + current_len) < (w as usize) {
                (w as usize).saturating_sub(right_art_w + 2 + right_label_len + 1)
            } else {
                current_len + 2
            };
            let right_start = if right_edge_start > desired_left_shift {
                let shifted = right_edge_start.saturating_sub(desired_left_shift);
                // ensure we never overlap the left block
                std::cmp::max(shifted, current_len + 2)
            } else {
                std::cmp::max(right_edge_start, current_len + 2)
            };

            // Add spacing up to right_start
            let mut acc_len = current_len;
            if right_start > acc_len {
                spans.push(Span::raw(" ".repeat(right_start - acc_len)));
                acc_len = right_start;
            }

            // Right art
            spans.push(Span::styled(right_art[row], right_style));
            spans.push(Span::raw("  "));

            // Right label only on middle row
            if row == 1 {
                spans.push(Span::styled(right_label.clone(), right_style));
            } else {
                spans.push(Span::raw(" ".repeat(right_label_len)));
            }

            lines.push(Line::from(spans));
        }

        // Legend / hint
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("<Tab> to switch islands, [Space] to select", Style::default().fg(Color::Gray)),
            Span::raw("   [m] Close map")
        ]));

        // Bottom animated wave with a little ship
        let mut bottom = String::new();
        let ship_pos = ((millis / 150) % (w as u128)) as usize;
        for i in 0..(w as usize) {
            if i == ship_pos {
                bottom.push('>');
            } else {
                let ch = wave_chars[(i + phase) % wave_chars.len()];
                bottom.push(ch);
            }
        }
        lines.push(Line::from(vec![Span::styled(bottom, Style::default().fg(Color::Blue))]));

        // Render panel
        let widget = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Island Map"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, area);
        f.render_widget(widget, area);
    }

    fn render_crafting_area(&self, f: &mut Frame, area: Rect) {
        let recipes = self.crafting.get_recipes();

        // Title line at the top of the crafting area
        let title = Paragraph::new(Line::from(vec![
            Span::styled("Crafting", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ]))
        .block(Block::default())
        .alignment(Alignment::Left);
        let title_rect = Rect::new(area.x, area.y, area.width, 1);
        f.render_widget(title, title_rect);

        // Split the remaining space into three compact columns with small gaps between them
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // left column
                Constraint::Length(2),      // spacer
                Constraint::Percentage(36), // middle column (slightly larger)
                Constraint::Length(2),      // spacer
                Constraint::Percentage(30), // right column
            ])
            .split(Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1)));

        // Prepare lines for each column
        let mut left_lines: Vec<Line> = Vec::new();
        let mut mid_lines: Vec<Line> = Vec::new();
        let mut right_lines: Vec<Line> = Vec::new();

        // Column headers (show hotkey hints)
        left_lines.push(Line::from(Span::styled("Tools [1]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        left_lines.push(Line::from(Span::raw("")));
        mid_lines.push(Line::from(Span::styled("Construction [2]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        mid_lines.push(Line::from(Span::raw("")));
        // Right column header becomes 'Actions' after Boat is built
        let right_header = if self.crafting.get_completed_items().iter().any(|it| it == "Boat") {
            "Actions"
        } else {
            "Locked"
        };
        right_lines.push(Line::from(Span::styled(format!("{} [3]", right_header), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        right_lines.push(Line::from(Span::raw("")));

        // Helper to render the craft sentence in one condensed line (shows typed progress)
        let render_sentence = |recipe: &crate::crafting::Recipe| -> Line {
            let mut spans = Vec::new();
            for (i, c) in recipe.craft_sentence.chars().enumerate() {
                let style = if i < recipe.current_input.len() {
                    if c == ' ' {
                        Style::default().fg(Color::Black).bg(Color::Green)
                    } else {
                        Style::default().fg(Color::Green)
                    }
                } else {
                    Style::default().fg(Color::Gray)
                };
                let ch = if c == ' ' { '·' } else { c };
                spans.push(Span::styled(ch.to_string(), style));
            }
            Line::from(spans)
        };

        // Distribute recipes into columns and render each as a compact 2-line tile
        for (idx, recipe) in recipes.iter().enumerate() {
            if !self.crafting.is_recipe_unlocked(idx) { continue; }

            let reqs = self.crafting.get_requirements_text(recipe);
            let mut header_spans = vec![
                Span::styled(&recipe.name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(reqs, Style::default().fg(Color::Blue)),
                Span::raw("  "),
                Span::raw(&recipe.description),
            ];
            if recipe.name.starts_with("Upgrade") && recipe.upgrade_count > 0 {
                header_spans.push(Span::raw("  "));
                header_spans.push(Span::styled(format!("(Lvl {})", recipe.upgrade_count + 1), Style::default().fg(Color::Yellow)));
            }

            let header_line = Line::from(header_spans);
            let sentence_line = render_sentence(recipe);

            // Decide column by recipe type/name
            // Treat `Iron Sword` as a Tools entry so it appears in the left column
            if recipe.name.starts_with("Upgrade") || recipe.name == "Iron Sword" {
                left_lines.push(header_line);
                left_lines.push(sentence_line);
                left_lines.push(Line::from(Span::raw("")));
            } else if recipe.name == "Workbench" || recipe.name == "Boat" {
                mid_lines.push(header_line);
                mid_lines.push(sentence_line);
                mid_lines.push(Line::from(Span::raw("")));
            } else {
                right_lines.push(header_line);
                right_lines.push(sentence_line);
                right_lines.push(Line::from(Span::raw("")));
            }
        }

        // Ensure each column has at least one placeholder line
        if left_lines.len() <= 2 { left_lines.push(Line::from(Span::raw("(no tools yet)"))); }
        if mid_lines.len() <= 2 { mid_lines.push(Line::from(Span::raw("(no structures yet)"))); }
        if right_lines.len() <= 2 { right_lines.push(Line::from(Span::raw("(locked slots)"))); }

        // Render columns
        let left_widget = Paragraph::new(left_lines).wrap(Wrap { trim: true });
        let mid_widget = Paragraph::new(mid_lines).wrap(Wrap { trim: true });
        let right_widget = Paragraph::new(right_lines).wrap(Wrap { trim: true });

        // If a section is expanded, render that section full-width; otherwise render three columns
        match self.crafting_expanded {
            1 => { f.render_widget(left_widget, Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1))); },
            2 => { f.render_widget(mid_widget, Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1))); },
            3 => { f.render_widget(right_widget, Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1))); },
            _ => {
                f.render_widget(left_widget, cols[0]);
                f.render_widget(mid_widget, cols[2]);
                f.render_widget(right_widget, cols[4]);
            }
        }
    }

    fn get_next_word(&self, resource_type: ResourceType) -> String {
        let difficulty = match resource_type {
            ResourceType::Wood => WordDifficulty::Easy,
            ResourceType::Copper => WordDifficulty::Medium,
            ResourceType::Iron => WordDifficulty::Medium,
        };
        self.word_list.get_random_word(difficulty).to_string()
    }

    fn replace_word(&mut self, idx: usize) {
        // First get the resource type and generate the new word
        let resource_type = self.resources.get(idx)
            .map(|r| r.resource_type.clone())
            .unwrap_or(ResourceType::Wood);
        let new_next = self.get_next_word(resource_type);
        
        // Then update the resource
        if let Some(resource) = self.resources.get_mut(idx) {
            resource.craft_sentence = resource.next_craft_sentence.clone();
            resource.next_craft_sentence = new_next;
            resource.current_input.clear();
        }
    }

    fn save_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Update session time before saving
        self.stats.update_session_time();

        // Validate that we have reasonable data before saving
        if self.player.wood > 1000 || self.player.copper > 1000 || self.player.iron > 1000 {
            // eprintln!("Warning: Unusual resource amounts detected, skipping save");
            return Ok(());
        }

        let save_data = SaveData {
            version: 1,
            player_wood: self.player.wood,
            player_copper: self.player.copper,
            player_iron: self.player.iron,
            player_level: self.player.level,
            player_xp: self.player.xp,
            completed_items: self.crafting.get_completed_items().to_vec(),
            has_workbench: self.crafting.has_workbench,
            has_boat: self.crafting.get_completed_items().iter().any(|it| it == "Boat"),
            has_iron_sword_unlocked: self.crafting.is_unlocked_by_name("Iron Sword"),
            current_island_index: self.island_manager.get_current_island_index() as u32,
            completed_quests: self.quest_manager.get_completed_quests(),
            axe_upgrade_count: self.crafting.get_recipes()
                .iter()
                .find(|r| r.name == "Upgrade Axe")
                .map(|r| r.upgrade_count)
                .unwrap_or(0),
            pickaxe_upgrade_count: self.crafting.get_recipes()
                .iter()
                .find(|r| r.name == "Upgrade Pickaxe")
                .map(|r| r.upgrade_count)
                .unwrap_or(0),
            stats: self.stats.clone(),
            save_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        };

        // Debug output to help track saves
        // println!("Saving: Wood={}, Copper={}", save_data.player_wood, save_data.player_copper);
        
        self.save_manager.save_game(&save_data)?;
        Ok(())
    }
}

fn append_log(msg: &str) {
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("keycrafter.log")
        .and_then(|mut f| writeln!(f, "{}", msg));
}

fn main() -> Result<(), Box<dyn Error>> {
    // Check for update argument
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "update" {
        return Updater::self_update();
    }

    // Regular game startup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create game state
    let mut game = Game::new();

    // Game loop with proper cleanup
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        loop {
            // Draw the UI
            if let Err(e) = terminal.draw(|f| ui(f, &mut game)) {
                append_log(&format!("Failed to draw UI: {}", e));
                break Err(e.into());
            }

            // Handle events
            if let Ok(true) = crossterm::event::poll(Duration::from_millis(50)) {
                match event::read() {
                    Ok(Event::Key(key)) => {
                        // Only process key press events, ignore releases
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::F(10) | KeyCode::Esc => {
                                    // Save before exiting
                                    let _ = game.save_game();
                                    break Ok(());
                                }
                                KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                    // Ctrl+Q for emergency exit
                                    let _ = game.save_game();
                                    break Ok(());
                                }
                                _ => {
                                    if let Some(_version_info) = game.handle_key(key) {
                                        // Update was requested, exit cleanly
                                        break Ok(());
                                    }
                                },
                            }
                        }
                    }
                    Ok(_) => {} // Ignore other events
                    Err(e) => {
                        append_log(&format!("Event read error: {}", e));
                        break Err(e.into());
                    }
                }
            }

            // Update game state
            game.update();
        }
    }));

    // Always restore terminal, even if there was a panic
    let cleanup_result = (|| -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    })();

    // Handle cleanup errors
    if let Err(cleanup_err) = cleanup_result {
        append_log(&format!("Failed to cleanup terminal: {}", cleanup_err));
    }

    // Handle the main result
    match result {
        Ok(game_result) => game_result,
        Err(_) => {
            append_log("Game panicked, but terminal should be restored");
            Err("Game panicked".into())
        }
    }
}

fn ui(f: &mut Frame, game: &mut Game) {
    let size = f.size();
    
    // Split screen into game area and crafting area (reduced game area to give more room to crafting)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(20),  // Game area (reduced)
            Constraint::Length(16), // Crafting area increased for more UI space
        ])
        .split(size);
    
    game.render_game_area(f, chunks[0]);
    game.render_crafting_area(f, chunks[1]);
}
    