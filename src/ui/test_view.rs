use std::thread::current;

use crate::core::typing_test::TypingTest;

// Pos of a single word on the screen
#[derive(Debug, Clone)]
pub struct WordLayout {
    pub word_index: usize,
    pub line_number: usize,
    pub x_position: usize,
}

// Dimensions and positioning of the typing area
#[derive(Debug, Clone)]
pub struct ViewportConfig {
    pub width: usize,
    pub visible_lines: usize,
    pub h_offset: u16,
}

impl ViewportConfig {
    pub fn new(terminal_width: u16, visible_lines: usize) -> Self {
        // NOTE: Padding is hard coded for now
        let width = ((terminal_width as f32) * 0.8) as usize;
        let h_offset: u16 = ((terminal_width as usize - width) / 2) as u16;

        Self {
            width,
            visible_lines,
            h_offset,
        }
    }
}

// Manages what is visible on screen
pub struct TestView {
    viewport: ViewportConfig,
    layouts: Vec<WordLayout>,
    visible_start_index: usize,
}

impl TestView {
    pub fn new(viewport: ViewportConfig) -> Self {
        Self {
            viewport,
            layouts: vec![WordLayout {
                word_index: 0,
                line_number: 4,
                x_position: 1,
            }],
            visible_start_index: 0,
        }
    }

    pub fn layouts(&self) -> &[WordLayout] {
        &self.layouts
    }

    // REcompute word positions based on test state
    // Returns true if we need to add more words cause its a new line
    pub fn update_layout(&mut self, test: &TypingTest) -> bool {
        let word_widths = self.compute_word_widths(test);
        let should_scroll = self.handle_scroll(test.current_word_index(), &word_widths);
        self.compute_layouts(&word_widths);
        should_scroll
    }

    fn compute_layouts(&mut self, word_widths: &[usize]) {
        self.layouts.clear();

        if self.visible_start_index >= word_widths.len() {
            return;
        }

        let mut current_line = 0;
        let mut current_x = 0;

        for (i, &width) in word_widths.iter().enumerate() {
            let word_index = self.visible_start_index + i;
            let padd_x = if current_x > 0 { 1 } else { 0 }; // Space between chars unless first
            let total_needed = current_x + padd_x + width;

            // Dont wrap a long first word
            if total_needed > self.viewport.width && current_x > 0 {
                current_line += 1;
                current_x = 0;
            }

            self.layouts.push(WordLayout {
                word_index,
                line_number: current_line,
                x_position: current_x,
            });

            current_x += width + 1;

            if current_line > self.viewport.visible_lines {
                break;
            }
        }
    }

    pub fn handle_scroll(&mut self, current_word_idx: usize, _word_widths: &[usize]) -> bool {
        let old_start = self.visible_start_index;

        // Scroll back to prev word if curr word has gone before the visible range (only if freedom mode on)
        if current_word_idx < self.visible_start_index {
            self.visible_start_index = current_word_idx;
            return true;
        }

        // Bit of a sliding window, if we are on next line we scroll
        let current_line = self
            .layouts
            .iter()
            .find_map(|l| (l.word_index == current_word_idx).then(|| l.line_number));

        if let Some(line) = current_line {
            if line >= 1 {
                if let Some(new_start) = self
                    .layouts
                    .iter()
                    .find_map(|l| (l.line_number == 1).then(|| l.word_index))
                {
                    self.visible_start_index = new_start;
                }
            }
        }

        self.visible_start_index != old_start
    }

    fn compute_word_widths(&mut self, test: &TypingTest) -> Vec<usize> {
        let curr_idx = test.current_word_index();

        test.words()
            .iter()
            .enumerate()
            .map(|(i, word)| match i.cmp(&curr_idx) {
                std::cmp::Ordering::Less => {
                    // User couldve typed less (still wanna show the full word) or more
                    let typed_len = word.typed.as_ref().map_or(0, |s| s.len());
                    std::cmp::max(typed_len, word.target.len())
                }
                std::cmp::Ordering::Equal => {
                    std::cmp::max(test.current_input().len(), word.target.len())
                } // Same idea but we get from what is typed
                std::cmp::Ordering::Greater => word.target.len(),
            })
            .collect()
    }
}
