use std::time::Duration;

/// every five characters count as one word
const CHARS_PER_WORD: f64 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// The expected character, or a space right at the end of a word
    Hit,
    /// Anything else
    Miss,
    /// A backspace into a correct word, which takes it off the score
    Erase,
}

#[derive(Clone, Copy, Debug)]
pub struct Keystroke {
    /// Time since the test started
    pub at: Duration,
    pub key: Key,
    /// The test score right after this keystroke
    pub score: usize,
}

/// How the final text compares against the target, character by character
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CharCounts {
    pub correct: usize,
    pub incorrect: usize,
    /// Typed past the end of a word
    pub extra: usize,
    /// Skipped over by pressing space early
    pub missed: usize,
}

/// One second of a test
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Second {
    /// Net WPM averaged over the whole test up to the end of this second
    pub wpm: f64,
    /// Raw WPM within this second alone
    pub raw: f64,
    pub errors: usize,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub wpm: f64,
    pub raw: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub duration: Duration,
    pub chars: CharCounts,
    pub timeline: Vec<Second>,
}

impl Report {
    /// no. of correctly typed 'words' spaces included
    pub fn new(log: &[Keystroke], score: usize, duration: Duration, chars: CharCounts) -> Self {
        let secs = duration.as_secs_f64();
        let hits = log.iter().filter(|k| k.key == Key::Hit).count();
        let misses = log.iter().filter(|k| k.key == Key::Miss).count();
        let typed = hits + misses;
        let timeline = timeline(log, secs);
        let raw: Vec<f64> = timeline.iter().map(|s| s.raw).collect();

        Self {
            wpm: wpm(score, secs),
            raw: wpm(typed, secs),
            accuracy: if typed == 0 {
                0.0
            } else {
                100.0 * hits as f64 / typed as f64
            },
            consistency: consistency(&raw),
            duration,
            chars,
            timeline,
        }
    }
}

fn wpm(chars: usize, secs: f64) -> f64 {
    if secs <= 0.0 {
        return 0.0;
    }
    chars as f64 / CHARS_PER_WORD * 60.0 / secs
}

/// Splits the log into 1s slices. floor merges
fn timeline(log: &[Keystroke], secs: f64) -> Vec<Second> {
    let slices = (secs.round() as usize).max(1);
    let mut keys = log.iter().peekable();
    let mut score = 0;

    (0..slices)
        .map(|i| {
            let last = i + 1 == slices;
            let end = if last { secs } else { (i + 1) as f64 };
            let (mut typed, mut errors) = (0, 0);

            while let Some(k) = keys.next_if(|k| last || k.at.as_secs_f64() < end) {
                score = k.score;
                match k.key {
                    Key::Hit => typed += 1,
                    Key::Miss => {
                        typed += 1;
                        errors += 1;
                    }
                    Key::Erase => {}
                }
            }

            Second {
                wpm: wpm(score, end),
                raw: wpm(typed, end - i as f64),
                errors,
            }
        })
        .collect()
}

/// How steady the pace was % wise
fn consistency(samples: &[f64]) -> f64 {
    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    if mean <= 0.0 {
        return 0.0;
    }

    let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let cv = variance.sqrt() / mean;
    100.0 * (1.0 - (cv + cv.powi(3) / 3.0 + cv.powi(5) / 5.0).tanh())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(secs: f64, key: Key, score: usize) -> Keystroke {
        Keystroke {
            at: Duration::from_secs_f64(secs),
            key,
            score,
        }
    }

    #[test]
    fn wpm_counts_five_chars_per_word() {
        assert_eq!(wpm(250, 60.0), 50.0);
        assert_eq!(wpm(10, 0.0), 0.0);
    }

    #[test]
    fn timeline_buckets_by_second() {
        let log = [
            key(0.2, Key::Hit, 0),
            key(0.4, Key::Miss, 0),
            key(1.5, Key::Hit, 5),
        ];
        let t = timeline(&log, 2.0);

        assert_eq!(t.len(), 2);
        assert_eq!(t[0].errors, 1);
        assert_eq!(t[0].raw, wpm(2, 1.0));
        assert_eq!(t[1].raw, wpm(1, 1.0));
        assert_eq!(t[1].wpm, wpm(5, 2.0));
    }

    #[test]
    fn timeline_folds_leftover_fraction_into_last_second() {
        let log = [key(0.5, Key::Hit, 1), key(1.9, Key::Hit, 2)];
        let t = timeline(&log, 2.3);

        assert_eq!(t.len(), 2);
        assert!((t[1].raw - wpm(1, 1.3)).abs() < 1e-9);
    }

    #[test]
    fn steady_pace_is_fully_consistent() {
        assert_eq!(consistency(&[60.0, 60.0, 60.0]), 100.0);
        assert_eq!(consistency(&[0.0, 0.0]), 0.0);
        assert!(consistency(&[20.0, 100.0, 40.0]) < 60.0);
    }

    #[test]
    fn accuracy_ignores_erases() {
        let log = [
            key(0.1, Key::Hit, 0),
            key(0.2, Key::Miss, 0),
            key(0.3, Key::Erase, 0),
            key(0.4, Key::Hit, 0),
            key(0.5, Key::Hit, 0),
        ];
        let report = Report::new(&log, 0, Duration::from_secs(1), CharCounts::default());

        assert_eq!(report.accuracy, 75.0);
    }
}
