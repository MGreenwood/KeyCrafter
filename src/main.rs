mod pathfinding;
mod ascii_objects;
mod floating_text;
mod upgrades;
mod islands;
mod resource_types;
mod crafting;
use pathfinding::{Grid, Position};
use ascii_objects::ResourceObjects;
use floating_text::FloatingTextManager;
use upgrades::UpgradeManager;
use islands::IslandManager;
use resource_types::ResourceType;
use crafting::CraftingManager;

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
    word: String,
    current_input: String,
    harvests_remaining: u32,
    max_harvests: u32,
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
}

impl Game {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let words = vec![
            "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
            "typing", "speed", "practice", "keyboard", "fingers",
            "craft", "build", "create", "develop", "program",
        ];
        
        // Pick two different random words
        let mut available_words = words.clone();
        let first_word_idx = rng.gen_range(0..available_words.len());
        let first_word = available_words.remove(first_word_idx);
        let second_word_idx = rng.gen_range(0..available_words.len());
        let second_word = available_words[second_word_idx].to_string();

        let resources = vec![
            {
                let (min_harvests, max_harvests) = ResourceType::Wood.get_base_harvests();
                let max_harvests = rng.gen_range(min_harvests..=max_harvests);
                Resource {
                    position: Position::new(15, 10),
                    resource_type: ResourceType::Wood,
                    word: first_word.to_string(),
                    current_input: String::new(),
                    harvests_remaining: max_harvests,
                    max_harvests,
                }
            },
            {
                let (min_harvests, max_harvests) = ResourceType::Copper.get_base_harvests();
                let max_harvests = rng.gen_range(min_harvests..=max_harvests);
                Resource {
                    position: Position::new(45, 10),
                    resource_type: ResourceType::Copper,
                    word: second_word,
                    current_input: String::new(),
                    harvests_remaining: max_harvests,
                    max_harvests,
                }
            },
        ];
        
        let mut grid = Grid::new();
        for resource in &resources {
            grid.add_obstacle(resource.position.clone());
        }
        
        Self {
            player: Player::new(10, 10),
            resources,
            last_update: Instant::now(),
            grid,
            resource_objects: ResourceObjects::new(),
            floating_texts: FloatingTextManager::new(),
            upgrades: UpgradeManager::new(),
            island_manager: IslandManager::new(),
            crafting: CraftingManager::new(),
            debug_messages: Vec::new(),
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
                
                // Add obstacles for the entire object area except the path point
                for dy in 0..h {
                    for dx in 0..w {
                        let pos = Position::new((rx + dx) as i32, (ry + dy) as i32);
                        if pos != target {  // Don't block the target position
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
                // Get existing positions
                let existing_positions: Vec<(i32, i32)> = self.resources
                    .iter()
                    .map(|r| (r.position.x, r.position.y))
                    .collect();

                // Try to find a spawn position
                if let Some((x, y)) = self.island_manager.find_spawn_position(&existing_positions, 80, 24) {
                    // Get random word for the new resource
                    let mut rng = rand::thread_rng();
                    let words = vec![
                        "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
                        "typing", "speed", "practice", "keyboard", "fingers",
                        "craft", "build", "create", "develop", "program",
                    ];
                    
                    // Get currently used words
                    let used_words: Vec<String> = self.resources
                        .iter()
                        .map(|r| r.word.clone())
                        .collect();
                    
                    // Filter out used words
                    let available_words: Vec<&str> = words
                        .iter()
                        .filter(|w| !used_words.contains(&w.to_string()))
                        .copied()
                        .collect();
                    
                    if !available_words.is_empty() {
                        let word = available_words[rng.gen_range(0..available_words.len())].to_string();
                        
                        // Create new resource
                        let resource_type = self.island_manager.get_random_resource_type();
                        
                        // Calculate random harvest limit
                        let (min_harvests, max_harvests) = resource_type.get_base_harvests();
                        let max_harvests = rng.gen_range(min_harvests..=max_harvests);
                        
                        let new_resource = Resource {
                            position: Position::new(x, y),
                            resource_type,
                            word,
                            current_input: String::new(),
                            harvests_remaining: max_harvests,
                            max_harvests,
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
    }

    fn harvest_resource(&mut self) {
        // Find resource at player's position that has a completed word
        if let Some(resource) = self.resources.iter_mut().find(|r| {
            r.position == self.player.position && 
            r.current_input == r.word
        }) {
            // Get the resource to check harvests remaining
            resource.harvests_remaining = resource.harvests_remaining.saturating_sub(1);
            
            // Update resources and show floating text
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
            
            // Show floating text
            self.floating_texts.add_text(
                format!("+{} {}", amount, text),
                self.player.position.x as f32,
                self.player.position.y as f32 - 1.0,
                color
            );
            
            // Clear the input
            resource.current_input.clear();
            
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
                let target_word = &resource.word;

                // If this letter matches the next expected letter for this word, add it
                if current_pos < target_word.len() && target_word.chars().nth(current_pos) == Some(c) {
                    // If we're just starting or continuing the word
                    resource.current_input.push(c);
                    
                    // If this is the first letter, calculate the path
                    if current_pos == 0 {
                        debug_messages.push(format!("Started word '{}' with '{}'", target_word, c));
                        debug_messages.push(format!("Player at {:?}, calculating path to {:?}", self.player.position, resource.position));
                        
                        // Clear and rebuild grid obstacles
                        self.grid.clear_obstacles();
                        for (pos, (rx, ry, w, h)) in &resource_obstacles {
                            if *pos != resource.position {  // Don't block target
                                // Add obstacles for the object area
                                for dy in 0..*h {
                                    for dx in 0..*w {
                                        self.grid.add_obstacle(Position::new((*rx + dx) as i32, (*ry + dy) as i32));
                                    }
                                }
                            }
                        }

                        if let Some(path) = self.grid.find_path(self.player.position.clone(), resource.position.clone()) {
                            debug_messages.push(format!("Found path with {} steps", path.len()));
                            self.player.path = path;
                            self.player.target = Some(resource.position.clone());
                        } else {
                            debug_messages.push("No path found!".to_string());
                        }
                    } else {
                        debug_messages.push(format!("Continued word '{}' with '{}'", target_word, c));
                    }

                    // Move one step for each correct letter
                    debug_messages.push(format!("Path has {} steps remaining", self.player.path.len()));
                    if !self.player.path.is_empty() {
                        let old_pos = self.player.position.clone();
                        self.player.position = self.player.path.remove(0);
                        debug_messages.push(format!("Moved from {:?} to {:?}", old_pos, self.player.position));
                    } else {
                        debug_messages.push("No steps left in path!".to_string());
                    }

                    // Check if we completed the word
                    if resource.current_input == *target_word {
                        let distance = self.player.position.manhattan_distance(&resource.position);
                        if distance <= 1 {
                            debug_messages.push("Word complete and close enough, marking for harvest".to_string());
                            should_harvest = true;
                        }
                        completed_word_idx = Some(resource_idx);
                    }
                }
                // If we've started typing this word but got a wrong letter, clear it
                else if current_pos > 0 {
                    debug_messages.push(format!("Wrong letter for '{}', clearing input", target_word));
                    resource.current_input.clear();
                    // Clear path when we make a mistake
                    self.player.path.clear();
                    self.player.target = None;
                }
            }

            // Handle harvest after the loop
            if should_harvest {
                debug_messages.push("Harvesting resource".to_string());
                self.harvest_resource();
                self.player.target = None;
                self.player.path.clear();
            }

            // Replace completed word with a new one
            if let Some(idx) = completed_word_idx {
                let mut rng = rand::thread_rng();
                let words = vec![
                    "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
                    "typing", "speed", "practice", "keyboard", "fingers",
                    "craft", "build", "create", "develop", "program",
                ];
                let word = words[rng.gen_range(0..words.len())].to_string();
                if let Some(resource) = self.resources.get_mut(idx) {
                    debug_messages.push(format!("Replacing completed word '{}' with '{}'", resource.word, word));
                    resource.word = word;
                    resource.current_input.clear();
                }
            }

            // Add all debug messages
            for message in debug_messages {
                self.add_debug_message(message);
            }
        }
    }

    fn render_game_area(&self, f: &mut Frame, game_area: Rect) {
        let mut lines = Vec::new();
        
        // Create empty grid
        for y in 0..game_area.height {
            let mut line_spans = Vec::new();
            for x in 0..game_area.width {
                let pos = Position::new(x as i32, y as i32);
                
                // Check if player is here
                // Check if this position contains player or resource
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
                                let word_start = rx.saturating_sub(resource.word.len() / 2);
                                let word_end = word_start + resource.word.len();
                                let x_pos = x as usize;
                                
                                if x_pos >= word_start && x_pos < word_end {
                                    let char_idx = x_pos - word_start;
                                    if let Some(c) = resource.word.chars().nth(char_idx) {
                                        let style = if char_idx < resource.current_input.len() {
                                            Style::default().fg(Color::Green)
                                        } else {
                                            Style::default().fg(Color::White)
                                        };
                                        word_span = Some(Span::styled(c.to_string(), style));
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
                    for (i, c) in recipe.word.chars().enumerate() {
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