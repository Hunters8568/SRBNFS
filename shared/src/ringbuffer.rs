#[derive(Clone)]
pub struct RingBuffer {
    data: Vec<String>,
    current_index: usize,
}

impl RingBuffer {
    pub fn new(data: Vec<std::string::String>) -> Self {
        Self {
            data,
            current_index: 0,
        }
    }

    pub fn next(&mut self) -> String {
        if self.current_index + 1 >= self.data.len() {
            self.current_index = 0;
        } else {
            self.current_index += 1;
        }

        return self.data[self.current_index].clone();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn at(&self, index: usize) -> String {
        self.data[index].clone()
    }
}
