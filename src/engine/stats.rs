/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗     ██╗    ██████╗     ██████╗
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝    ███║   ██╔═████╗   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║       ╚██║   ██║██╔██║   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║        ██║   ████╔╝██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║        ██║██╗╚██████╔╝██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝        ╚═╝╚═╝ ╚═════╝ ╚═╝ ╚═════╝

Made with ♥ by tfaullk


*/

use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,
    pub incorrect_keystrokes: usize,
    pub elapsed: Duration,
    // one wpm reading per second, used for the consistency score
    pub wpm_samples: Vec<f64>,
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_keystrokes == 0 {
            return 100.0;
        }
        (self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0
    }

    // standard 5-chars-per-word convention
    pub fn wpm(&self) -> f64 {
        let minutes = self.elapsed.as_secs_f64() / 60.0;
        if minutes <= 0.0 {
            return 0.0;
        }
        (self.correct_keystrokes as f64 / 5.0) / minutes
    }

    // same thing but counting every keystroke, mistakes included
    pub fn raw_wpm(&self) -> f64 {
        let minutes = self.elapsed.as_secs_f64() / 60.0;
        if minutes <= 0.0 {
            return 0.0;
        }
        (self.total_keystrokes as f64 / 5.0) / minutes
    }

    // consistency = how steady your wpm was, based on the coefficient of variation
    pub fn consistency(&self) -> f64 {
        if self.wpm_samples.len() < 2 {
            return 0.0;
        }
        let mean: f64 = self.wpm_samples.iter().sum::<f64>() / self.wpm_samples.len() as f64;
        if mean == 0.0 {
            return 0.0;
        }
        let variance: f64 = self
            .wpm_samples
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / self.wpm_samples.len() as f64;
        let stddev = variance.sqrt();
        let cv = stddev / mean;
        (1.0 - cv).max(0.0) * 100.0
    }

    pub fn push_wpm_sample(&mut self, wpm: f64) {
        self.wpm_samples.push(wpm);
    }
}
