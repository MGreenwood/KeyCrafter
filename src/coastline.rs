use std::time::Instant;
use ratatui::style::{Color, Style};

pub struct Coastline {
    last_update: Instant,
    wave_frame: usize,
    wave_chars: Vec<&'static str>,
}

impl Coastline {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            wave_frame: 0,
            wave_chars: vec!["~", "≈", "~", "≈"],
        }
    }

    pub fn update(&mut self) {
        if self.last_update.elapsed().as_millis() > 500 {  // Update every 500ms
            self.wave_frame = (self.wave_frame + 1) % self.wave_chars.len();
            self.last_update = Instant::now();
        }
    }

    pub fn get_tile(&self, x: i32, y: i32, width: i32, height: i32) -> (String, Style) {
        // Define the island shape - oval/circle
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Make the land area large enough for gameplay (about 70% of the screen)
        let radius_x = (width * 3 / 5) as f32;  // Increased from width/3 to width*3/5
        let radius_y = (height * 3 / 5) as f32; // Increased from height/3 to height*3/5

        // Calculate distance from center (normalized)
        let dx = (x - center_x) as f32 / radius_x;
        let dy = (y - center_y) as f32 / radius_y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Adjust thresholds to make the land area larger
        // Land is now everything within 1.0 of the radius (was 0.8)
        // Coast is between 1.0 and 1.3 (was 0.8 and 1.2)
        if distance > 1.0 && distance < 1.3 {
            // Wave animation for water tiles
            let wave_char = self.wave_chars[self.wave_frame];
            (wave_char.to_string(), Style::default().fg(Color::Cyan))
        } else if distance <= 1.0 {
            // Land tiles - blank spaces for cleaner look
            (" ".to_string(), Style::default())
        } else {
            // Deep water
            let wave_char = self.wave_chars[self.wave_frame];
            (wave_char.to_string(), Style::default().fg(Color::Blue))
        }
    }
} 