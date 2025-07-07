use std::collections::HashMap;
use crate::resource_types::ResourceType;

#[derive(Clone, Debug)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub craft_sentence: String,  // A thematic sentence about crafting this item
    pub current_input: String,  // Current typing progress
    pub requirements: HashMap<ResourceType, u32>,
    pub unlocks: Vec<String>,  // Names of things this unlocks (other recipes, upgrades, etc.)
    pub upgrade_count: u32,  // Track how many times this upgrade has been completed
}

pub struct CraftingManager {
    recipes: Vec<Recipe>,
    unlocked_recipes: Vec<bool>,  // Parallel vec to track what's unlocked
    pub has_workbench: bool,  // Track if workbench has been crafted
    completed_items: Vec<String>,  // Track completed one-time items
}

impl CraftingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            recipes: Vec::new(),
            unlocked_recipes: Vec::new(),
            has_workbench: false,
            completed_items: Vec::new(),
        };

        // Add initial recipe - Workbench
        let mut workbench_reqs = HashMap::new();
        workbench_reqs.insert(ResourceType::Wood, 15);
        workbench_reqs.insert(ResourceType::Copper, 10);
        
        manager.recipes.push(Recipe {
            name: "Workbench".to_string(),
            description: "A basic crafting station. Unlocks new recipes.".to_string(),
            craft_sentence: "I carefully assemble wooden planks and copper joints to build a sturdy workbench.".to_string(),
            current_input: String::new(),
            requirements: workbench_reqs,
            unlocks: vec!["Advanced Tools".to_string()],
            upgrade_count: 0,
        });
        manager.unlocked_recipes.push(true);  // Workbench is initially available

        // Add workbench-dependent recipes (initially locked)
        
        // Upgrade Axe
        let mut axe_reqs = HashMap::new();
        axe_reqs.insert(ResourceType::Wood, 20);
        axe_reqs.insert(ResourceType::Copper, 15);
        manager.recipes.push(Recipe {
            name: "Upgrade Axe".to_string(),
            description: "+1 Wood per harvest".to_string(),
            craft_sentence: "I sharpen my axe blade and reinforce the handle for better wood harvesting.".to_string(),
            current_input: String::new(),
            requirements: axe_reqs,
            unlocks: vec![],
            upgrade_count: 0,
        });
        manager.unlocked_recipes.push(false);  // Locked until workbench is built

        // Upgrade Pickaxe
        let mut pickaxe_reqs = HashMap::new();
        pickaxe_reqs.insert(ResourceType::Wood, 15);
        pickaxe_reqs.insert(ResourceType::Copper, 20);
        manager.recipes.push(Recipe {
            name: "Upgrade Pickaxe".to_string(),
            description: "+1 Copper per harvest".to_string(),
            craft_sentence: "I forge a stronger pickaxe head and balance it for efficient mining.".to_string(),
            current_input: String::new(),
            requirements: pickaxe_reqs,
            unlocks: vec![],
            upgrade_count: 0,
        });
        manager.unlocked_recipes.push(false);  // Locked until workbench is built

        manager
    }

    pub fn get_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    pub fn get_recipe_mut(&mut self, index: usize) -> Option<&mut Recipe> {
        self.recipes.get_mut(index)
    }

    pub fn is_recipe_unlocked(&self, index: usize) -> bool {
        if index == 0 {  // Workbench is special
            !self.has_workbench  // Only show if not yet crafted
        } else {
            self.has_workbench && self.unlocked_recipes.get(index).copied().unwrap_or(false)
        }
    }

    pub fn get_completed_items(&self) -> &[String] {
        &self.completed_items
    }

    pub fn load_from_save(&mut self, save_data: &crate::save_system::SaveData) {
        self.has_workbench = save_data.has_workbench;
        self.completed_items = save_data.completed_items.clone();
        
        // Restore upgrade counts
        if let Some(axe_recipe) = self.recipes.iter_mut().find(|r| r.name == "Upgrade Axe") {
            axe_recipe.upgrade_count = save_data.axe_upgrade_count;
        }
        if let Some(pickaxe_recipe) = self.recipes.iter_mut().find(|r| r.name == "Upgrade Pickaxe") {
            pickaxe_recipe.upgrade_count = save_data.pickaxe_upgrade_count;
        }
        
        // Update unlocked recipes based on workbench status
        if self.has_workbench {
            for i in 1..self.unlocked_recipes.len() {
                self.unlocked_recipes[i] = true;
            }
        }
    }

    pub fn can_craft(&self, recipe_index: usize, wood: u32, copper: u32) -> bool {
        if let Some(recipe) = self.recipes.get(recipe_index) {
            // Check if recipe is unlocked
            if !self.is_recipe_unlocked(recipe_index) {
                return false;
            }

            // Check if we have enough resources
            for (resource_type, amount) in &recipe.requirements {
                match resource_type {
                    ResourceType::Wood if wood < *amount => return false,
                    ResourceType::Copper if copper < *amount => return false,
                    _ => {}
                }
            }
            true
        } else {
            false
        }
    }

    pub fn get_requirements_text(&self, recipe: &Recipe) -> String {
        let mut parts = Vec::new();
        for (resource_type, amount) in &recipe.requirements {
            parts.push(format!("{} {}", amount, resource_type.get_display_name()));
        }
        parts.join(" + ")
    }

    pub fn craft_item(&mut self, recipe_index: usize) -> Option<(Recipe, HashMap<ResourceType, u32>)> {
        // Get lengths before any mutable borrow
        let recipes_len = self.recipes.len();
        let unlocked_len = self.unlocked_recipes.len();
        let mut crafted_name = None;
        let mut crafted_idx = None;

        if let Some(recipe) = self.recipes.get_mut(recipe_index) {
            // Check if the sentence is fully typed
            if recipe.current_input == recipe.craft_sentence {
                // Save debug info
                crafted_name = Some(recipe.name.clone());
                crafted_idx = Some(recipe_index);
                // If this is the workbench, unlock workbench-dependent recipes
                if recipe_index == 0 {
                    self.has_workbench = true;
                    self.completed_items.push("Workbench".to_string());
                    // Unlock all workbench-dependent recipes, but only if the vectors are in sync
                    if unlocked_len == recipes_len {
                        for i in 1..self.unlocked_recipes.len() {
                            self.unlocked_recipes[i] = true;
                        }
                    } else {
                        println!("[ERROR] unlocked_recipes and recipes length mismatch: {} vs {}", unlocked_len, recipes_len);
                    }
                } else {
                    // For upgrades, increment the upgrade count
                    recipe.upgrade_count += 1;
                }
                // Clear the input after crafting
                recipe.current_input.clear();
                // Return a clone of the recipe and its costs
                let result = Some((recipe.clone(), recipe.requirements.clone()));
                // Debug print after crafting (after mutable borrow ends)
                if let (Some(name), Some(idx)) = (crafted_name, crafted_idx) {
                    println!("[DEBUG] Crafted {} at index {}", name, idx);
                }
                return result;
            }
        }
        None
    }

    pub fn handle_input(&mut self, recipe_index: usize, c: char) -> bool {
        if let Some(recipe) = self.recipes.get_mut(recipe_index) {
            let target_sentence = &recipe.craft_sentence;
            let current_pos = recipe.current_input.len();

            // Prevent out-of-bounds access
            if current_pos >= target_sentence.len() {
                return false;
            }

            if target_sentence.chars().nth(current_pos) == Some(c) {
                recipe.current_input.push(c);
                true
            } else {
                // Always clear this recipe's input on wrong letter, but do not block others
                recipe.current_input.clear();
                false
            }
        } else {
            false
        }
    }

    pub fn clear_input(&mut self, recipe_index: usize) {
        if let Some(recipe) = self.recipes.get_mut(recipe_index) {
            recipe.current_input.clear();
        }
    }

    // Get the current multiplier for a resource type
    pub fn get_multiplier(&self, resource_type: &ResourceType) -> f32 {
        match resource_type {
            ResourceType::Wood => {
                // Get upgrade count from Axe upgrades
                if let Some(recipe) = self.recipes.iter().find(|r| r.name == "Upgrade Axe") {
                    1.0 + recipe.upgrade_count as f32  // Base of 1 + 1 for each upgrade
                } else {
                    1.0
                }
            },
            ResourceType::Copper => {
                // Get upgrade count from Pickaxe upgrades
                if let Some(recipe) = self.recipes.iter().find(|r| r.name == "Upgrade Pickaxe") {
                    1.0 + recipe.upgrade_count as f32  // Base of 1 + 1 for each upgrade
                } else {
                    1.0
                }
            },
        }
    }

    // Get the next cost for an upgrade recipe
    pub fn get_next_upgrade_cost(&self, recipe_index: usize) -> HashMap<ResourceType, u32> {
        let mut increased_costs = HashMap::new();
        if let Some(recipe) = self.recipes.get(recipe_index) {
            // Use the upgrade count to determine cost increase
            let crafted_count = recipe.upgrade_count;
            
            // Increase costs by 50% for each previous craft
            for (resource, &base_cost) in &recipe.requirements {
                let increased = base_cost + (base_cost * crafted_count) / 2;
                increased_costs.insert(resource.clone(), increased);
            }
        }
        increased_costs
    }
} 