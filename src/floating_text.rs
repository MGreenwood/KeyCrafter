use std::time::{Duration, Instant};

pub struct FloatingText {
    text: String,
    x: f32,
    y: f32,
    velocity_y: f32,
    color: ratatui::style::Color,
    created_at: Instant,
    lifetime: Duration,
}

impl FloatingText {
    pub fn new(text: String, x: f32, y: f32, color: ratatui::style::Color) -> Self {
        Self {
            text,
            x,
            y,
            velocity_y: 4.0,  // Move downward
            color,
            created_at: Instant::now(),
            lifetime: Duration::from_millis(800),  // Show for 0.8 seconds
        }
    }

    pub fn update(&mut self, dt: Duration) {
        // Move text downward
        let dt_secs = dt.as_secs_f32();
        self.y += self.velocity_y * dt_secs;
        
        // Slow down the downward movement
        self.velocity_y *= 0.90;  // Slow down faster
    }

    pub fn is_alive(&self) -> bool {
        self.created_at.elapsed() < self.lifetime
    }

    pub fn get_position(&self) -> (usize, usize) {
        (self.x.round() as usize, self.y.round() as usize)
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn get_color(&self) -> ratatui::style::Color {
        self.color
    }

    pub fn get_alpha(&self) -> f32 {
        // Fade out over lifetime
        let elapsed = self.created_at.elapsed();
        let remaining = (self.lifetime.as_secs_f32() - elapsed.as_secs_f32()) / self.lifetime.as_secs_f32();
        remaining.max(0.0).min(1.0)
    }
}

pub struct FloatingTextManager {
    texts: Vec<FloatingText>,
    last_update: Instant,
}

impl FloatingTextManager {
    pub fn new() -> Self {
        Self {
            texts: Vec::new(),
            last_update: Instant::now(),
        }
    }

    pub fn add_text(&mut self, text: String, x: f32, y: f32, color: ratatui::style::Color) {
        self.texts.push(FloatingText::new(text, x, y, color));
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update);
        self.last_update = now;

        // Update all texts
        for text in &mut self.texts {
            text.update(dt);
        }

        // Remove dead texts
        self.texts.retain(|text| text.is_alive());
    }

    pub fn get_texts(&self) -> &[FloatingText] {
        &self.texts
    }
} 