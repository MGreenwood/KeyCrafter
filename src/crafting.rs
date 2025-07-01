use std::collections::HashMap;
use crate::resource_types::ResourceType;

#[derive(Clone, Debug)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub word: String,  // The word to type to craft this item
    pub current_input: String,  // Current typing progress
    pub requirements: HashMap<ResourceType, u32>,
    pub unlocks: Vec<String>,  // Names of things this unlocks (other recipes, upgrades, etc.)
}

pub struct CraftingManager {
    recipes: Vec<Recipe>,
    unlocked_recipes: Vec<bool>,  // Parallel vec to track what's unlocked
}

impl CraftingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            recipes: Vec::new(),
            unlocked_recipes: Vec::new(),
        };

        // Add initial recipe - Workbench
        let mut workbench_reqs = HashMap::new();
        workbench_reqs.insert(ResourceType::Wood, 50);
        workbench_reqs.insert(ResourceType::Copper, 20);
        
        manager.recipes.push(Recipe {
            name: "Workbench".to_string(),
            description: "A basic crafting station. Unlocks new recipes.".to_string(),
            word: "construct_workbench".to_string(),  // Descriptive word about what you're crafting
            current_input: String::new(),
            requirements: workbench_reqs,
            unlocks: vec!["Advanced Tools".to_string()],  // Will unlock more recipes later
        });
        manager.unlocked_recipes.push(true);  // Workbench is initially available

        manager
    }

    pub fn get_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    pub fn get_recipe_mut(&mut self, index: usize) -> Option<&mut Recipe> {
        self.recipes.get_mut(index)
    }

    pub fn is_recipe_unlocked(&self, index: usize) -> bool {
        self.unlocked_recipes.get(index).copied().unwrap_or(false)
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
        if let Some(recipe) = self.recipes.get(recipe_index) {
            // Check if the word is fully typed
            if recipe.current_input == recipe.word {
                // Return a clone of the recipe and its costs
                return Some((recipe.clone(), recipe.requirements.clone()));
            }
        }
        None
    }

    pub fn handle_input(&mut self, recipe_index: usize, c: char) -> bool {
        if let Some(recipe) = self.recipes.get_mut(recipe_index) {
            let target_word = &recipe.word;
            let current_pos = recipe.current_input.len();
            
            if current_pos < target_word.len() {
                if target_word.chars().nth(current_pos) == Some(c) {
                    recipe.current_input.push(c);
                    true
                } else {
                    recipe.current_input.clear();
                    false
                }
            } else {
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
} 