#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResourceType {
    Wood,
    Copper,
    // Future types can be added here
}

impl ResourceType {
    pub fn get_base_harvests(&self) -> (u32, u32) {  // Returns (min, max) harvests
        match self {
            ResourceType::Wood => (6, 10),        // Trees have more harvests
            ResourceType::Copper => (4, 7),    // Copper has fewer harvests
        }
    }

    pub fn get_display_name(&self) -> &'static str {
        match self {
            ResourceType::Wood => "Wood",
            ResourceType::Copper => "Copper",
        }
    }

    pub fn get_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            ResourceType::Wood => Color::Green,
            ResourceType::Copper => Color::Yellow,
        }
    }

    pub fn get_symbol(&self) -> &'static str {
        match self {
            ResourceType::Wood => "/\\",  // Tree symbol
            ResourceType::Copper => "Cu",  // Copper symbol
        }
    }
} 