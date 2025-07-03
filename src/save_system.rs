use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::resource_types::ResourceType;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameStats {
    pub words_typed: u32,
    pub characters_typed: u32,
    pub resources_harvested: HashMap<ResourceType, u32>,
    pub total_play_time_seconds: u64,
    pub session_start_time: u64,
    pub words_completed: u32,
    pub crafting_attempts: u32,
    pub successful_crafts: u32,
    pub mistakes_made: u32,
    pub fastest_word_time: Option<f32>,
    pub average_wpm: f32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            words_typed: 0,
            characters_typed: 0,
            resources_harvested: HashMap::new(),
            total_play_time_seconds: 0,
            session_start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            words_completed: 0,
            crafting_attempts: 0,
            successful_crafts: 0,
            mistakes_made: 0,
            fastest_word_time: None,
            average_wpm: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveData {
    pub version: u32,
    pub player_wood: u32,
    pub player_copper: u32,
    pub completed_items: Vec<String>,
    pub has_workbench: bool,
    pub axe_upgrade_count: u32,
    pub pickaxe_upgrade_count: u32,
    pub stats: GameStats,
    pub save_timestamp: u64,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: 1,
            player_wood: 0,
            player_copper: 0,
            completed_items: Vec::new(),
            has_workbench: false,
            axe_upgrade_count: 0,
            pickaxe_upgrade_count: 0,
            stats: GameStats::default(),
            save_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        }
    }
}

pub struct SaveManager {
    save_file_path: String,
    backup_file_path: String,
    auto_save_interval: Duration,
    last_save_time: SystemTime,
}

impl SaveManager {
    pub fn new() -> Self {
        Self {
            save_file_path: "keycrafter_save.json".to_string(),
            backup_file_path: "keycrafter_save.backup.json".to_string(),
            auto_save_interval: Duration::from_secs(30), // Auto-save every 30 seconds
            last_save_time: SystemTime::now(),
        }
    }

    pub fn should_auto_save(&self) -> bool {
        SystemTime::now()
            .duration_since(self.last_save_time)
            .unwrap_or(Duration::ZERO)
            >= self.auto_save_interval
    }

    pub fn save_game(&mut self, save_data: &SaveData) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(save_data)?;

        // First, try to backup the existing save if it exists
        if Path::new(&self.save_file_path).exists() {
            if let Ok(existing_save) = fs::read_to_string(&self.save_file_path) {
                if let Ok(existing_data) = serde_json::from_str::<SaveData>(&existing_save) {
                    // Only backup if the existing save has more resources
                    if existing_data.player_wood > save_data.player_wood || 
                       existing_data.player_copper > save_data.player_copper {
                        fs::write(&self.backup_file_path, existing_save)?;
                        // println!("Created backup of save with more resources");
                    }
                }
            }
        }

        // Write the new save file
        fs::write(&self.save_file_path, &json)?;
        
        // Verify the save was written correctly
        let read_back = fs::read_to_string(&self.save_file_path)?;
        if read_back != json {
            return Err("Save file verification failed".into());
        }

        self.last_save_time = SystemTime::now();
        Ok(())
    }

    pub fn load_game(&self) -> Result<SaveData, Box<dyn std::error::Error>> {
        let mut save_data = SaveData::default();
        let mut loaded = false;

        // Try to load the main save file
        if Path::new(&self.save_file_path).exists() {
            if let Ok(json) = fs::read_to_string(&self.save_file_path) {
                if let Ok(data) = serde_json::from_str(&json) {
                    save_data = data;
                    loaded = true;
                }
            }
        }

        // If main save failed or has no resources, try the backup
        if (!loaded || (save_data.player_wood == 0 && save_data.player_copper == 0)) && 
           Path::new(&self.backup_file_path).exists() {
            if let Ok(json) = fs::read_to_string(&self.backup_file_path) {
                if let Ok(backup_data) = serde_json::from_str(&json) {
                    // Use backup if it has more resources
                    let backup_data: SaveData = backup_data;
                    if backup_data.player_wood > save_data.player_wood || 
                       backup_data.player_copper > save_data.player_copper {
                        // println!("Loaded backup save with more resources");
                        save_data = backup_data;
                    }
                }
            }
        }

        Ok(save_data)
    }

    pub fn delete_save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(&self.save_file_path).exists() {
            fs::remove_file(&self.save_file_path)?;
        }
        if Path::new(&self.backup_file_path).exists() {
            fs::remove_file(&self.backup_file_path)?;
        }
        Ok(())
    }
}

impl GameStats {
    pub fn update_session_time(&mut self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        let session_duration = current_time.saturating_sub(self.session_start_time);
        self.total_play_time_seconds += session_duration;
        self.session_start_time = current_time;
    }

    pub fn add_resource_harvested(&mut self, resource_type: ResourceType, amount: u32) {
        *self.resources_harvested.entry(resource_type).or_insert(0) += amount;
    }

    pub fn add_word_completed(&mut self, word_length: u32, time_taken: f32) {
        self.words_completed += 1;
        self.words_typed += 1;
        self.characters_typed += word_length;

        // Update fastest word time
        if self.fastest_word_time.is_none() || time_taken < self.fastest_word_time.unwrap() {
            self.fastest_word_time = Some(time_taken);
        }

        // Update average WPM (simplified calculation)
        if self.total_play_time_seconds > 0 {
            let minutes = self.total_play_time_seconds as f32 / 60.0;
            self.average_wpm = self.words_completed as f32 / minutes;
        }
    }

    pub fn add_mistake(&mut self) {
        self.mistakes_made += 1;
    }

    pub fn add_crafting_attempt(&mut self) {
        self.crafting_attempts += 1;
    }

    pub fn add_successful_craft(&mut self) {
        self.successful_crafts += 1;
    }

    pub fn get_accuracy_percentage(&self) -> f32 {
        if self.characters_typed == 0 {
            return 100.0;
        }
        let correct_chars = self.characters_typed.saturating_sub(self.mistakes_made);
        (correct_chars as f32 / self.characters_typed as f32) * 100.0
    }

    pub fn get_total_play_time_formatted(&self) -> String {
        let hours = self.total_play_time_seconds / 3600;
        let minutes = (self.total_play_time_seconds % 3600) / 60;
        let seconds = self.total_play_time_seconds % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
} 