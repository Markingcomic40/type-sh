pub struct TestView {
    layouts: Vec<WordLayout>,
}

pub struct WordLayout {
    pub word_index: usize,
    pub line_number: usize,
    pub x_position: usize,
}

impl TestView {
    pub fn new() -> Self {
        Self {
            layouts: vec![WordLayout {
                word_index: 0,
                line_number: 4,
                x_position: 1,
            }],
        }
    }

    pub fn layouts(&self) -> &[WordLayout] {
        &self.layouts
    }
}
