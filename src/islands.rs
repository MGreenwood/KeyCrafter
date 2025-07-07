use rand::Rng;
use crate::resource_types::ResourceType;

pub struct Island {
    pub name: String,
    pub resource_pools: Vec<ResourcePool>,
    pub max_nodes: u32,
    pub spawn_chance: f32,  // 0.0 to 1.0
    pub level_requirement: u32,
}

pub struct ResourcePool {
    pub resource_type: ResourceType,
    pub weight: u32,  // Higher weight = more likely to spawn
}

pub struct IslandManager {
    islands: Vec<Island>,
    current_island: usize,
}

impl IslandManager {
    pub fn new() -> Self {
        let mut manager = Self {
            islands: Vec::new(),
            current_island: 0,
        };

        // Starting island - Basic resources
        manager.islands.push(Island {
            name: "Starter Grove".to_string(),
            resource_pools: vec![
                ResourcePool {
                    resource_type: ResourceType::Wood,
                    weight: 60,
                },
                ResourcePool {
                    resource_type: ResourceType::Copper,
                    weight: 50,
                },
            ],
            max_nodes: 6,
            spawn_chance: 0.15,  // 15% chance per harvest
            level_requirement: 0,
        });

        // Future islands can be added here
        // Example:
        // manager.islands.push(Island {
        //     name: "Iron Mountains".to_string(),
        //     resource_pools: vec![
        //         ResourcePool { resource_type: ResourceType::Iron, weight: 50 },
        //         ResourcePool { resource_type: ResourceType::Copper, weight: 30 },
        //         ResourcePool { resource_type: ResourceType::Wood, weight: 20 },
        //     ],
        //     max_nodes: 6,
        //     spawn_chance: 0.4,
        //     level_requirement: 5,
        // });

        manager
    }

    pub fn get_current_island(&self) -> &Island {
        &self.islands[self.current_island]
    }

    pub fn should_spawn_node(&self) -> bool {
        let island = self.get_current_island();
        let mut rng = rand::thread_rng();
        let roll = rng.gen::<f32>();
        roll < island.spawn_chance
    }

    pub fn get_random_resource_type(&self) -> ResourceType {
        let island = self.get_current_island();
        let mut rng = rand::thread_rng();
        
        // Calculate total weight
        let total_weight: u32 = island.resource_pools.iter().map(|p| p.weight).sum();
        
        // Get random value
        let mut value = rng.gen_range(0..total_weight);
        
        // Find corresponding resource
        for pool in &island.resource_pools {
            if value < pool.weight {
                return pool.resource_type.clone();
            }
            value -= pool.weight;
        }
        
        // Fallback to first resource
        island.resource_pools[0].resource_type.clone()
    }

    pub fn find_spawn_position(&self, existing_positions: &[(i32, i32)], width: i32, height: i32) -> Option<(i32, i32)> {
        let mut rng = rand::thread_rng();
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 100;

        // Calculate the safe spawn area (within the land)
        let center_x = width / 2;
        let center_y = height / 2;
        let radius_x = (width * 3 / 5) as f32;
        let radius_y = (height * 3 / 5) as f32;

        while attempts < MAX_ATTEMPTS {
            // Generate random position
            let x = rng.gen_range(4..width-4);  // Leave margin for ASCII art
            let y = rng.gen_range(4..height-4);

            // Check if position is within the land area
            let dx = (x - center_x) as f32 / radius_x;
            let dy = (y - center_y) as f32 / radius_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Only allow spawning within 90% of the land radius to keep away from coast
            if distance <= 0.9 {
                // Check if position is far enough from existing nodes
                let is_valid = existing_positions.iter().all(|(ex, ey)| {
                    let dx = (x - ex).abs();
                    let dy = (y - ey).abs();
                    dx > 6 || dy > 4  // Minimum distance between nodes
                });

                if is_valid {
                    return Some((x, y));
                }
            }

            attempts += 1;
        }

        None
    }
} 