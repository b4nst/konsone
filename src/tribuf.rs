use serde::{Deserialize, Serialize};

/// Buffer is a naive implementation of a circular buffer of size 3,
/// with a single cursor.
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

    /// Push a value to the buffer.
    pub fn push(&mut self, value: T) {
        self.data[self.cursor] = value;
        self.cursor = (self.cursor + 1) % 3;
    }

    /// Return a vector of the buffer's non empty data, from newest to oldest.
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        for i in (self.cursor..self.cursor + 3).rev() {
            result.push(self.data[i % 3].clone());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buf = Buffer::<i32>::new();
        buf.push(1);
        assert_eq!(buf.to_vec(), vec![1, 0, 0]);
        buf.push(2);
        assert_eq!(buf.to_vec(), vec![2, 1, 0]);
        buf.push(3);
        buf.push(4);
        buf.push(5);
        buf.push(6);
        assert_eq!(buf.to_vec(), vec![6, 5, 4]);
    }
}
