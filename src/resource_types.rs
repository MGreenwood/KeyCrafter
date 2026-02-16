use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    Wood,
    Copper,
    Iron,
    // Future types can be added here
}

impl ResourceType {
    pub fn get_base_harvests(&self) -> (u32, u32) {  // Returns (min, max) harvests
        match self {
            ResourceType::Wood => (6, 10),        // Trees have more harvests
            ResourceType::Copper => (4, 7),    // Copper has fewer harvests
            ResourceType::Iron => (3, 5),      // Iron is moderate/rarer
        }
    }

    pub fn get_display_name(&self) -> &'static str {
        match self {
            ResourceType::Wood => "Wood",
            ResourceType::Copper => "Copper",
            ResourceType::Iron => "Iron",
        }
    }

    pub fn get_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            ResourceType::Wood => Color::Green,
            ResourceType::Copper => Color::Yellow,
            ResourceType::Iron => Color::Gray,
        }
    }

    pub fn get_symbol(&self) -> &'static str {
        match self {
            ResourceType::Wood => "/\\",  // Tree symbol
            ResourceType::Copper => "Cu",  // Copper symbol
            ResourceType::Iron => "Fe",    // Iron symbol
        }
    }
} 