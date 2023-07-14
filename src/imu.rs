

pub struct State {
    pub generation: usize
}

impl State {
    pub fn step(&self) -> Self {
        Self {
            generation: self.generation + 1
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            generation: 0
        }
    }
}
