use rand::seq::SliceRandom;

// Embed word lists at compile time
const EASY_WORDS: &str = include_str!("../resources/words_easy.txt");
const MEDIUM_WORDS: &str = include_str!("../resources/words_medium.txt");
const HARD_WORDS: &str = include_str!("../resources/words_hard.txt");

pub struct WordList {
    easy: Vec<&'static str>,
    medium: Vec<&'static str>,
    hard: Vec<&'static str>,
}

impl WordList {
    pub fn new() -> Self {
        Self {
            easy: EASY_WORDS.lines().map(|s| s.trim()).collect(),
            medium: MEDIUM_WORDS.lines().map(|s| s.trim()).collect(),
            hard: HARD_WORDS.lines().map(|s| s.trim()).collect(),
        }
    }

    pub fn get_random_word(&self, difficulty: WordDifficulty) -> &'static str {
        let mut rng = rand::thread_rng();
        match difficulty {
            WordDifficulty::Easy => self.easy.choose(&mut rng).unwrap_or(&"tree"),
            WordDifficulty::Medium => self.medium.choose(&mut rng).unwrap_or(&"copper"),
            WordDifficulty::Hard => self.hard.choose(&mut rng).unwrap_or(&"program"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum WordDifficulty {
    Easy,   // 3-4 letters
    Medium, // 5-6 letters
    Hard,   // 7+ letters
} 