use std::collections::HashMap;

pub struct AsciiObject {
    art: Vec<String>,
    width: usize,
    height: usize,
    path_point: (usize, usize),  // Where to path to within the object
}

impl AsciiObject {
    pub fn new(art: Vec<&str>, path_point: (usize, usize)) -> Self {
        let height = art.len();
        let width = art.iter().map(|line| line.len()).max().unwrap_or(0);
        
        // Pad lines to make all objects the same width
        let art = art.into_iter()
            .map(|line| {
                let mut line = line.to_string();
                while line.len() < width {
                    line.push(' ');
                }
                line
            })
            .collect();

        Self {
            art,
            width,
            height,
            path_point,
        }
    }

    pub fn render_at(&self, x: usize, y: usize) -> Vec<(usize, usize, char)> {
        let mut chars = Vec::new();
        for (dy, line) in self.art.iter().enumerate() {
            for (dx, c) in line.chars().enumerate() {
                if c != ' ' {
                    chars.push((x + dx, y + dy, c));
                }
            }
        }
        chars
    }

    pub fn get_path_point(&self, x: usize, y: usize) -> (usize, usize) {
        (x + self.path_point.0, y + self.path_point.1)
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

pub struct ResourceObjects {
    objects: HashMap<String, AsciiObject>,
}

impl ResourceObjects {
    pub fn new() -> Self {
        let mut objects = HashMap::new();

        // Tree
        objects.insert("tree".to_string(), AsciiObject::new(vec![
            " /\\ ",
            "/~~\\",
            " || ",
            " || ",
        ], (2, 3))); // Path to bottom of trunk

        // Copper Ore
        objects.insert("copper".to_string(), AsciiObject::new(vec![
            " /\\ ",
            "(Cu)",
            "\\__/",
            " || ",
        ], (2, 3))); // Path to bottom

        // Iron Ore (for future use)
        objects.insert("iron".to_string(), AsciiObject::new(vec![
            " /\\ ",
            "(Fe)",
            "\\__/",
            " || ",
        ], (2, 3)));

        // Gold Ore (for future use)
        objects.insert("gold".to_string(), AsciiObject::new(vec![
            " /\\ ",
            "(Au)",
            "\\__/",
            " || ",
        ], (2, 3)));

        // Herb (for future use)
        objects.insert("herb".to_string(), AsciiObject::new(vec![
            " () ",
            "\\||/",
            " \\/ ",
            " || ",
        ], (2, 3)));

        Self { objects }
    }

    pub fn get(&self, name: &str) -> Option<&AsciiObject> {
        self.objects.get(name)
    }

    pub fn add(&mut self, name: String, art: Vec<&str>, path_point: (usize, usize)) {
        self.objects.insert(name, AsciiObject::new(art, path_point));
    }
} 