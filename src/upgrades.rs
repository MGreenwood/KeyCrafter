use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Upgrade {
    pub name: String,
    pub description: String,
    pub cost_type: ResourceType,
    pub cost_amount: u32,
    pub effect: UpgradeEffect,
    pub level: u32,
}

use crate::resource_types::ResourceType;

#[derive(Clone, Debug)]
pub enum UpgradeEffect {
    WoodMultiplier(f32),
    CopperMultiplier(f32),
}

pub struct UpgradeManager {
    upgrades: Vec<Upgrade>,
    resource_multipliers: HashMap<ResourceType, f32>,
}

impl UpgradeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            upgrades: Vec::new(),
            resource_multipliers: HashMap::new(),
        };

        // Initialize base multipliers
        manager.resource_multipliers.insert(ResourceType::Wood, 1.0);
        manager.resource_multipliers.insert(ResourceType::Copper, 1.0);

        // Add initial upgrades
        manager.upgrades.push(Upgrade {
            name: "Better Axe".to_string(),
            description: "+1 Wood per harvest".to_string(),
            cost_type: ResourceType::Wood,
            cost_amount: 10,
            effect: UpgradeEffect::WoodMultiplier(1.0),
            level: 0,
        });

        manager.upgrades.push(Upgrade {
            name: "Better Pickaxe".to_string(),
            description: "+1 Copper per harvest".to_string(),
            cost_type: ResourceType::Copper,
            cost_amount: 10,
            effect: UpgradeEffect::CopperMultiplier(1.0),
            level: 0,
        });

        manager
    }

    pub fn get_upgrades(&self) -> &[Upgrade] {
        &self.upgrades
    }

    pub fn get_multiplier(&self, resource_type: &ResourceType) -> f32 {
        *self.resource_multipliers.get(resource_type).unwrap_or(&1.0)
    }

    pub fn can_purchase(&self, upgrade_index: usize, wood: u32, copper: u32) -> bool {
        if let Some(upgrade) = self.upgrades.get(upgrade_index) {
            let cost = self.get_next_cost(upgrade);
            match upgrade.cost_type {
                ResourceType::Wood => wood >= cost,
                ResourceType::Copper => copper >= cost,
            }
        } else {
            false
        }
    }

    pub fn get_next_cost(&self, upgrade: &Upgrade) -> u32 {
        // Cost increases exponentially with level
        upgrade.cost_amount + (upgrade.level as u32 * 5)
    }

    pub fn purchase_upgrade(&mut self, upgrade_index: usize) -> Option<u32> {
        // Get cost before mutable borrow
        let cost = if let Some(upgrade) = self.upgrades.get(upgrade_index) {
            self.get_next_cost(upgrade)
        } else {
            return None;
        };

        if let Some(upgrade) = self.upgrades.get_mut(upgrade_index) {
            
            // Update multiplier
            match upgrade.effect {
                UpgradeEffect::WoodMultiplier(amount) => {
                    *self.resource_multipliers.get_mut(&ResourceType::Wood).unwrap() += amount;
                }
                UpgradeEffect::CopperMultiplier(amount) => {
                    *self.resource_multipliers.get_mut(&ResourceType::Copper).unwrap() += amount;
                }
            }

            // Increase level
            upgrade.level += 1;

            Some(cost)
        } else {
            None
        }
    }

    pub fn format_resource_amount(&self, amount: u32, resource_type: &ResourceType) -> String {
        match resource_type {
            ResourceType::Wood => format!("{} Wood", amount),
            ResourceType::Copper => format!("{} Copper", amount),
        }
    }
} 