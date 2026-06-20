#[derive(Clone, Debug)]
pub struct TestResults {
    pub wpm: f32,
    pub raw_wpm: f32,
    pub accuracy: f32,
    pub consistency: f32,
    pub correct_chars: usize,
    pub total_chars: usize,
    pub time_elapsed: f32,
}

pub struct Statistics {
    correct_chars: usize,
    total_chars: usize,
    wpm_samples: Vec<f32>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            correct_chars: 0,
            total_chars: 0,
            wpm_samples: Vec::new(),
        }
    }

    fn calc_wpm(&self, elapsed_secs: f32) -> f32 {
        if elapsed_secs <= 0.0 {
            return 0.0;
        }
        (self.correct_chars as f32 * 60.0) / (5.0 * elapsed_secs)
    }

    fn calc_raw_wpm(&self, elapsed_secs: f32) -> f32 {
        if elapsed_secs <= 0.0 {
            return 0.0;
        }
        (self.total_chars as f32 * 60.0) / (5.0 * elapsed_secs)
    }

    fn calc_accuracy(&self) -> f32 {
        if self.total_chars == 0 {
            return 0.0;
        }
        (self.correct_chars as f32 / self.total_chars as f32) * 100.0
    }

    fn calc_consistency(&self) -> f32 {
        let n = self.wpm_samples.len();
        if n < 2 {
            return 100.0;
        }

        let mean = self.wpm_samples.iter().sum::<f32>() / n as f32;
        if mean <= 0.0 {
            return 0.0;
        }

        let variance = self
            .wpm_samples
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>()
            / n as f32;

        let std_dev = variance.sqrt();
        (100.0 - (std_dev / mean * 100.0)).clamp(0.0, 100.0)
    }

    pub fn record_char(&mut self, is_correct: bool) {
        self.total_chars += 1;

        if is_correct {
            self.correct_chars += 1;
        }
    }

    pub fn into_results(self, elapsed_secs: f32) -> TestResults {
        TestResults {
            wpm: self.calc_wpm(elapsed_secs),
            raw_wpm: self.calc_raw_wpm(elapsed_secs),
            accuracy: self.calc_accuracy(),
            consistency: self.calc_consistency(),
            correct_chars: self.correct_chars,
            total_chars: self.total_chars,
            time_elapsed: elapsed_secs,
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}
