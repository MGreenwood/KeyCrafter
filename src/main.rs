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
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Self {
            position: Position::new(x, y),
            path: Vec::new(),
            target: None,
            wood: 0,
            copper: 0,
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
    updater: Updater,
    pending_update: Option<VersionInfo>,
    coastline: Coastline,
}

impl Game {
    fn new() -> Self {
        let save_manager = SaveManager::new();
        let save_data = save_manager.load_game().unwrap_or_default();
        
        let mut rng = rand::thread_rng();
        let word_list = WordList::new();
        let mut resources = Vec::new();
        let island_manager = IslandManager::new();
        
        // Start with half the max nodes
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
        
        // Debug output to help track loads
        // println!("Loaded: Wood={}, Copper={}", player.wood, player.copper);
        
        // Create crafting manager and load saved state
        let mut crafting = CraftingManager::new();
        crafting.load_from_save(&save_data);
        
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
            updater: Updater::new(),
            pending_update: None,
            coastline: Coastline::new(),
        };
        
        // ... rest of initialization ...
        game
    }
    
    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(50) {
            // Update floating texts
            self.floating_texts.update();
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

        match key.code {
            KeyCode::Char('u') if self.pending_update.is_some() => {
                // Clone version info before any mutable borrow
                let version_info = self.pending_update.as_ref().cloned();
                if let Some(version_info) = version_info {
                    let _ = self.save_game();
                    match self.updater.download_update(&version_info) {
                        Ok(new_exe_path) => {
                            if let Err(e) = self.updater.apply_update(&new_exe_path) {
                                eprintln!("Failed to apply update: {}", e);
                            } else {
                                // Signal that we want to exit for update
                                return Some(version_info);
                            }
                        }
                        Err(e) => eprintln!("Failed to download update: {}", e),
                    }
                }
            }
            KeyCode::Char(c) => {
                // Handle crafting input - check all recipes simultaneously
                let mut crafting_completed = false;
                let mut completed_recipe_idx = None;
                let mut any_crafting_progress = false;
                
                for recipe_idx in 0..self.crafting.get_recipes().len() {
                    if self.crafting.is_recipe_unlocked(recipe_idx) && 
                       self.crafting.can_craft(recipe_idx, self.player.wood, self.player.copper) {
                        if self.crafting.handle_input(recipe_idx, c) {
                            any_crafting_progress = true;
                            // Check if crafting is complete
                            if let Some((recipe, costs)) = self.crafting.craft_item(recipe_idx) {
                                self.stats.add_successful_craft();
                                
                                // Deduct resources
                                for (resource_type, amount) in costs {
                                    match resource_type {
                                        ResourceType::Wood => self.player.wood -= amount,
                                        ResourceType::Copper => self.player.copper -= amount,
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
                                }
                                
                                crafting_completed = true;
                                completed_recipe_idx = Some(recipe_idx);
                            } else {
                                // Track crafting attempt (typing in progress)
                                self.stats.add_crafting_attempt();
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

                // First collect all resource positions and their obstacles
                let mut resource_obstacles = Vec::new();
                for resource in &self.resources {
                    let obj = match resource.resource_type {
                        ResourceType::Wood => self.resource_objects.get("tree"),
                        ResourceType::Copper => self.resource_objects.get("copper"),
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
                                }
                                resource.word_start_time = None;
                                
                                // Get the target position
                                let target_pos = if let Some(obj) = match resource.resource_type {
                                    ResourceType::Wood => self.resource_objects.get("tree"),
                                    ResourceType::Copper => self.resource_objects.get("copper"),
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
                if y == 0 && x >= game_area.width.saturating_sub(30) {
                    if x == game_area.width.saturating_sub(30) {
                        let wood_text = format!("Wood: {}", self.player.wood);
                        let copper_text = format!("Copper: {}", self.player.copper);
                        line_spans.push(Span::styled(
                            wood_text,
                            Style::default().fg(ResourceType::Wood.get_color())
                        ));
                        line_spans.push(Span::raw(" | "));
                        line_spans.push(Span::styled(
                            copper_text,
                            Style::default().fg(ResourceType::Copper.get_color())
                        ));
                        // Skip the rest of this line
                        break;
                    }
                    continue;
                }
                
                // Add completed items display on the right side
                if x >= game_area.width.saturating_sub(25) {
                    let right_area_x = x as usize - (game_area.width.saturating_sub(25) as usize);
                    
                    // Show completed items (starting from line 2)
                    let completed_items = self.crafting.get_completed_items();
                    for (item_idx, item) in completed_items.iter().enumerate() {
                        let item_line = 2 + item_idx as u16;
                        if y == item_line && right_area_x < item.len() {
                            if let Some(c) = item.chars().nth(right_area_x) {
                                line_spans.push(Span::styled(
                                    c.to_string(),
                                    Style::default().fg(Color::Green)
                                ));
                                continue;
                            }
                        }
                    }
                    
                    // Fill with spaces if nothing to show
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
            .block(Block::default().borders(Borders::ALL).title("KeyCrafter - Island 1"));
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

        // Show debug info at the bottom if enabled
        if self.show_debug_info {
            let debug_text = format!("Loaded: Wood={}, Copper={}", self.player.wood, self.player.copper);
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

    fn render_crafting_area(&self, f: &mut Frame, area: Rect) {
        let recipes = self.crafting.get_recipes();
        let mut crafting_text = Vec::new();

        // Title
        crafting_text.push(Line::from(vec![
            Span::styled("Crafting", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ]));
        crafting_text.push(Line::from(""));

        // Display each recipe
        for (idx, recipe) in recipes.iter().enumerate() {
            if self.crafting.is_recipe_unlocked(idx) {
                // Recipe name and description
                let mut name_spans = vec![
                    Span::styled(&recipe.name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" - "),
                    Span::raw(&recipe.description)
                ];
                
                // Add upgrade level if this is an upgrade recipe
                if recipe.name.starts_with("Upgrade") {
                    if recipe.upgrade_count > 0 {
                        name_spans.push(Span::raw(" ("));
                        name_spans.push(Span::styled(
                            format!("Level {}", recipe.upgrade_count + 1),
                            Style::default().fg(Color::Yellow)
                        ));
                        name_spans.push(Span::raw(")"));
                    }
                }
                
                crafting_text.push(Line::from(name_spans));

                // Requirements
                let requirements = self.crafting.get_requirements_text(recipe);
                crafting_text.push(Line::from(vec![
                    Span::styled(format!("Requires: {}", requirements), Style::default().fg(Color::Blue))
                ]));

                // Crafting progress
                if !recipe.current_input.is_empty() {
                    let mut progress_spans = Vec::new();
                    for (i, c) in recipe.craft_sentence.chars().enumerate() {
                        let style = if i < recipe.current_input.len() {
                            if c == ' ' {
                                // Show spaces as green background with a visible character
                                Style::default().fg(Color::Black).bg(Color::Green)
                            } else {
                                Style::default().fg(Color::Green)
                            }
                        } else {
                            Style::default().fg(Color::Gray)
                        };
                        
                        let display_char = if c == ' ' && i < recipe.current_input.len() {
                            "▓".to_string() // Use a block character to make the space visible
                        } else if c == ' ' {
                            "·".to_string() // Use a middle dot to show untyped spaces
                        } else {
                            c.to_string()
                        };
                        
                        progress_spans.push(Span::styled(display_char, style));
                    }
                    crafting_text.push(Line::from(progress_spans));
                } else {
                    // Show the sentence with visible space indicators
                    let mut display_spans = Vec::new();
                    display_spans.push(Span::styled("Type to craft: ", Style::default().fg(Color::Gray)));
                    
                    for c in recipe.craft_sentence.chars() {
                        let display_char = if c == ' ' {
                            "·".to_string() // Show spaces as middle dots when not started
                        } else {
                            c.to_string()
                        };
                        display_spans.push(Span::styled(display_char, Style::default().fg(Color::Gray)));
                    }
                    
                    crafting_text.push(Line::from(display_spans));
                }

                // Add a blank line between recipes
                crafting_text.push(Line::from(""));
            }
        }

        let crafting_paragraph = Paragraph::new(crafting_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Crafting"))
            .wrap(Wrap { trim: true });

        f.render_widget(crafting_paragraph, area);
    }

    fn get_next_word(&self, resource_type: ResourceType) -> String {
        let difficulty = match resource_type {
            ResourceType::Wood => WordDifficulty::Easy,
            ResourceType::Copper => WordDifficulty::Medium,
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
        if self.player.wood > 1000 || self.player.copper > 1000 {
            // eprintln!("Warning: Unusual resource amounts detected, skipping save");
            return Ok(());
        }

        let save_data = SaveData {
            version: 1,
            player_wood: self.player.wood,
            player_copper: self.player.copper,
            completed_items: self.crafting.get_completed_items().to_vec(),
            has_workbench: self.crafting.has_workbench,
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
                eprintln!("Failed to draw UI: {}", e);
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
                        eprintln!("Event read error: {}", e);
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
        eprintln!("Failed to cleanup terminal: {}", cleanup_err);
    }

    // Handle the main result
    match result {
        Ok(game_result) => game_result,
        Err(_) => {
            eprintln!("Game panicked, but terminal should be restored");
            Err("Game panicked".into())
        }
    }
}

fn ui(f: &mut Frame, game: &mut Game) {
    let size = f.size();
    
    // Split screen into game area and crafting area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(24),  // Game area
            Constraint::Length(12), // Increased crafting area height
        ])
        .split(size);
    
    game.render_game_area(f, chunks[0]);
    game.render_crafting_area(f, chunks[1]);
}