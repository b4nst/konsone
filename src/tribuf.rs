use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Buffer<T> {
    data: [T; 3],
    cursor: usize,
}

impl<T: std::default::Default + Clone> Buffer<T> {
    pub fn new() -> Self {
        Self {
            data: [Default::default(), Default::default(), Default::default()],
            cursor: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.data[self.cursor] = value;
        self.cursor = (self.cursor + 1) % 3;
    }

    // Function ngram returns a vector of the buffer's non empty data, from newest to oldest.
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        for i in (self.cursor..self.cursor + 3).rev() {
            result.push(self.data[i % 3].clone());
        }
        result
    }
}
