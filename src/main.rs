mod pathfinding;
mod ascii_objects;
mod floating_text;
mod upgrades;
mod islands;
mod resource_types;
mod crafting;
mod word_lists;

use pathfinding::{Grid, Position};
use ascii_objects::ResourceObjects;
use floating_text::FloatingTextManager;
use upgrades::UpgradeManager;
use islands::IslandManager;
use resource_types::ResourceType;
use crafting::CraftingManager;
use word_lists::{WordList, WordDifficulty};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
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
    debug_messages: Vec<String>,
    word_list: WordList,
}

impl Game {
    fn new() -> Self {
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
                };
                
                resources.push(new_resource);
            }
        }
        
        let mut grid = Grid::new();
        for resource in &resources {
            grid.add_obstacle(resource.position.clone());
        }
        
        // Start player in middle of screen
        let player = Player::new(40, 12);
        
        Self {
            player,
            resources,
            last_update: Instant::now(),
            grid,
            resource_objects: ResourceObjects::new(),
            floating_texts: FloatingTextManager::new(),
            upgrades: UpgradeManager::new(),
            island_manager,
            crafting: CraftingManager::new(),
            debug_messages: Vec::new(),
            word_list,
        }
    }
    
    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(50) {
            // Update floating texts
            self.floating_texts.update();
            self.last_update = now;
        }
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
                        let multiplier = self.upgrades.get_multiplier(&ResourceType::Wood);
                        let amount = (multiplier as u32).max(1);
                        self.player.wood += amount;
                        (amount, "Wood".to_string(), ResourceType::Wood.get_color())
                    },
                    ResourceType::Copper => {
                        let multiplier = self.upgrades.get_multiplier(&ResourceType::Copper);
                        let amount = (multiplier as u32).max(1);
                        self.player.copper += amount;
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
    
    fn add_debug_message(&mut self, message: String) {
        self.debug_messages.push(message);
        // Keep only the last 20 messages
        if self.debug_messages.len() > 20 {
            self.debug_messages.remove(0);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if let KeyCode::Char(c) = key.code {
            let mut debug_messages = Vec::new();
            debug_messages.push(format!("Key pressed: {}", c));
            let mut should_harvest = false;
            let mut completed_word_idx = None;

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
                
                debug_messages.push(format!(">>> WORD CHECK: '{}' (current input: '{}', pos: {})", 
                    target_word, resource.current_input, current_pos));

                // If we haven't started this word yet, check if this is the first letter
                if current_pos == 0 {
                    let expected = target_word.chars().next();
                    debug_messages.push(format!(">>> FIRST LETTER CHECK: Expected '{}', got '{}'", 
                        expected.unwrap_or('?'), c));
                    
                    if expected == Some(c) {
                        // Start this word
                        resource.current_input.push(c);
                        debug_messages.push(format!(">>> STARTED: New word '{}' with '{}'", target_word, c));
                        
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

                        debug_messages.push(format!(">>> PATH: From {:?} to {:?}", self.player.position, target_pos));
                        
                        // Clear and rebuild grid obstacles
                        self.grid.clear_obstacles();
                        let mut obstacle_count = 0;
                        for (pos, (rx, ry, w, h)) in &resource_obstacles {
                            if *pos != resource.position {  // Don't block target
                                // Add obstacles for the object area
                                for dy in 0..*h {
                                    for dx in 0..*w {
                                        let obstacle_pos = Position::new((*rx + dx) as i32, (*ry + dy) as i32);
                                        if obstacle_pos != target_pos {  // Don't block the actual target point
                                            self.grid.add_obstacle(obstacle_pos);
                                            obstacle_count += 1;
                                        }
                                    }
                                }
                            }
                        }
                        debug_messages.push(format!("Added {} obstacles to grid", obstacle_count));

                        if let Some(path) = self.grid.find_path(self.player.position.clone(), target_pos.clone()) {
                            debug_messages.push(format!(">>> PATH: Found {} steps", path.len()));
                            resource.path = path;  // Store path in the resource
                            self.player.target = Some(target_pos);
                        } else {
                            debug_messages.push(">>> PATH: No path found!".to_string());
                        }

                        // Move first step
                        if !resource.path.is_empty() {
                            let old_pos = self.player.position.clone();
                            self.player.position = resource.path.remove(0);
                            debug_messages.push(format!(">>> MOVE: From {:?} to {:?} ({} steps left)", 
                                old_pos, self.player.position, resource.path.len()));
                        }
                    }
                }
                // If we've started this word, continue it
                else {
                    let expected = target_word.chars().nth(current_pos);
                    debug_messages.push(format!(">>> CONTINUE CHECK: Expected '{}', got '{}'", 
                        expected.unwrap_or('?'), c));
                    
                    if expected == Some(c) {
                        // Continue the word
                        resource.current_input.push(c);
                        debug_messages.push(format!(">>> MATCHED: Added '{}' to word. Now: '{}'", 
                            c, resource.current_input));

                        // Move one step
                        if !resource.path.is_empty() {
                            let old_pos = self.player.position.clone();
                            self.player.position = resource.path.remove(0);
                            debug_messages.push(format!(">>> MOVE: From {:?} to {:?} ({} steps left)", 
                                old_pos, self.player.position, resource.path.len()));
                        }

                        // Check if word is complete
                        if resource.current_input == *target_word {
                            debug_messages.push(format!(">>> COMPLETE: Word '{}' finished!", target_word));
                            completed_word_idx = Some(resource_idx);
                            
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
                            debug_messages.push(format!(">>> HARVEST CHECK: Distance = {}, Position = {:?}, Target = {:?}", 
                                distance, self.player.position, target_pos));

                            if distance <= 2 {
                                debug_messages.push(">>> HARVEST: Close enough to harvest!".to_string());
                                should_harvest = true;
                            } else {
                                debug_messages.push(">>> HARVEST: Too far to harvest yet, continuing to move".to_string());
                                // Clear any existing path and calculate new path to target
                                resource.path.clear();
                                if let Some(path) = self.grid.find_path(self.player.position.clone(), target_pos.clone()) {
                                    debug_messages.push(format!(">>> PATH: {} steps to get in range", path.len()));
                                    resource.path = path;
                                    self.player.target = Some(target_pos);
                                    // Move one step along the new path immediately
                                    if !resource.path.is_empty() {
                                        let old_pos = self.player.position.clone();
                                        self.player.position = resource.path.remove(0);
                                        debug_messages.push(format!(">>> MOVE: From {:?} to {:?} ({} steps left)", 
                                            old_pos, self.player.position, resource.path.len()));
                                    }
                                }
                            }
                        }
                    } else {
                        // Wrong letter, clear this word
                        debug_messages.push(format!(">>> WRONG: Expected '{}', got '{}'. Clearing word '{}'", 
                            expected.unwrap_or('?'), c, target_word));
                        resource.current_input.clear();
                        resource.path.clear();
                    }
                }
            }

            // Handle harvest after the loop
            if should_harvest {
                debug_messages.push(">>> EXECUTING HARVEST".to_string());
                self.harvest_resource();
                self.player.target = None;
            }

            // Replace completed word with a new one
            if let Some(idx) = completed_word_idx {
                debug_messages.push(format!(">>> REPLACING completed word at index {}", idx));
                self.replace_word(idx);
            }

            // Add all debug messages
            self.debug_messages.extend(debug_messages);
        }
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
                        
                        word_span.unwrap_or_else(|| Span::raw(" "))
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
            if y < game_area.height as usize && x < game_area.width as usize {
                // Get what's currently at this position
                let current_line = &lines[y];
                let mut new_line = current_line.spans.clone();
                
                // Calculate where in the line to insert the text
                let text = floating_text.get_text();
                let start_x = x.min(game_area.width as usize - text.len());
                
                // Replace spans at the text position
                for (i, c) in text.chars().enumerate() {
                    if start_x + i < game_area.width as usize {
                        let color = floating_text.get_color();
                        new_line[start_x + i] = Span::styled(
                            c.to_string(),
                            Style::default().fg(color).add_modifier(Modifier::BOLD)
                        );
                    }
                }
                
                // Render just this line
                let text_pos = Rect::new(
                    0,
                    y as u16,
                    game_area.width,
                    1,
                );
                let text_widget = Paragraph::new(Line::from(new_line));
                f.render_widget(text_widget, text_pos);
            }
        }
    }

    fn render_crafting_area(&self, f: &mut Frame, area: Rect) {
        let mut crafting_text = Vec::new();
        
        // Show available recipes
        for (i, recipe) in self.crafting.get_recipes().iter().enumerate() {
            if self.crafting.is_recipe_unlocked(i) {
                let key = (b'a' + i as u8) as char;
                let can_craft = self.crafting.can_craft(i, self.player.wood, self.player.copper);
                
                let mut spans = Vec::new();
                
                // Add key with appropriate color
                spans.push(Span::styled(
                    format!("{}. ", key),
                    if can_craft {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }
                ));
                
                // Add recipe name and requirements
                spans.push(Span::styled(
                    format!("{} ({})", recipe.name, self.crafting.get_requirements_text(recipe)),
                    if can_craft {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }
                ));
                
                // Show typing progress
                if !recipe.current_input.is_empty() {
                    spans.push(Span::raw(" | "));
                    for (i, c) in recipe.craft_sentence.chars().enumerate() {
                        spans.push(Span::styled(
                            c.to_string(),
                            if i < recipe.current_input.len() {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            }
                        ));
                    }
                }
                
                crafting_text.push(Line::from(spans));
            }
        }
        
        let crafting_widget = Paragraph::new(crafting_text)
            .block(Block::default().borders(Borders::ALL).title("Crafting & Upgrades"));
        f.render_widget(crafting_widget, area);
    }

    fn render_debug_area(&self, f: &mut Frame, area: Rect) {
        let debug_text: Vec<Line> = self.debug_messages.iter()
            .map(|msg| Line::from(vec![
                Span::styled(msg, Style::default().fg(Color::Gray))
            ]))
            .collect();

        let debug_widget = Paragraph::new(debug_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Debug Info"))
            .wrap(Wrap { trim: true });

        f.render_widget(debug_widget, area);
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
            self.debug_messages.push(format!(">>> WORD REPLACE: '{}' -> '{}', next will be '{}'", 
                resource.craft_sentence, resource.next_craft_sentence, new_next));
            resource.craft_sentence = resource.next_craft_sentence.clone();
            resource.next_craft_sentence = new_next;
            resource.current_input.clear();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create game state
    let mut game = Game::new();

    // Game loop
    let result = loop {
        // Draw the UI
        terminal.draw(|f| ui(f, &mut game))?;

        // Handle events
        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    // Only process key press events, ignore releases
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::F(10) | KeyCode::Esc => {
                                break Ok(());
                            }
                            _ => game.handle_key(key),
                        }
                    }
                }
                // Ignore all other events (mouse, resize, focus, paste)
                _ => {}
            }
        }

        // Update game state
        game.update();
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn ui(f: &mut Frame, game: &mut Game) {
    let size = f.size();
    
    // Calculate layout with debug area
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(80), // Main game area
            Constraint::Length(40), // Debug area
        ])
        .split(size);
    
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(24),  // Game area
            Constraint::Length(8), // Crafting area
        ])
        .split(chunks[0]);
    
    game.render_game_area(f, main_chunks[0]);
    game.render_crafting_area(f, main_chunks[1]);
    game.render_debug_area(f, chunks[1]);
}