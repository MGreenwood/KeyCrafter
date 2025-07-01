use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn manhattan_distance(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn neighbors(&self) -> Vec<Position> {
        vec![
            Position::new(self.x + 1, self.y),
            Position::new(self.x - 1, self.y),
            Position::new(self.x, self.y + 1),
            Position::new(self.x, self.y - 1),
        ]
        .into_iter()
        .filter(|p| p.x >= 0 && p.x < 80 && p.y >= 0 && p.y < 80)
        .collect()
    }
}

#[derive(Eq, PartialEq)]
struct Node {
    position: Position,
    f_score: i32,
    g_score: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Grid {
    obstacles: HashSet<Position>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            obstacles: HashSet::new(),
        }
    }

    pub fn add_obstacle(&mut self, pos: Position) {
        self.obstacles.insert(pos);
    }

    pub fn clear_obstacles(&mut self) {
        self.obstacles.clear();
    }

    pub fn is_walkable(&self, pos: &Position) -> bool {
        !self.obstacles.contains(pos)
    }

    pub fn find_path(&self, start: Position, goal: Position) -> Option<Vec<Position>> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from: std::collections::HashMap<Position, Position> = std::collections::HashMap::new();
        let mut g_scores = std::collections::HashMap::new();

        // Initialize start node
        g_scores.insert(start.clone(), 0);
        open_set.push(Node {
            position: start.clone(),
            f_score: 0,
            g_score: 0,
        });

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                // Reconstruct path
                let mut path = vec![goal.clone()];
                let mut current_pos = goal;
                while let Some(prev) = came_from.get(&current_pos) {
                    path.push(prev.clone());
                    current_pos = prev.clone();
                }
                path.reverse();
                return Some(path);
            }

            if !closed_set.insert(current.position.clone()) {
                continue;
            }

            for neighbor in current.position.neighbors() {
                if !self.is_walkable(&neighbor) || closed_set.contains(&neighbor) {
                    continue;
                }

                let tentative_g_score = current.g_score + 1;
                let neighbor_g_score = g_scores.get(&neighbor).copied().unwrap_or(i32::MAX);

                if tentative_g_score < neighbor_g_score {
                    came_from.insert(neighbor.clone(), current.position.clone());
                    g_scores.insert(neighbor.clone(), tentative_g_score);
                    let f_score = tentative_g_score + neighbor.manhattan_distance(&goal);
                    
                    open_set.push(Node {
                        position: neighbor,
                        f_score,
                        g_score: tentative_g_score,
                    });
                }
            }
        }

        None
    }
} 