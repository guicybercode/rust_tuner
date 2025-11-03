use rustfft::{FftPlanner, num_complex::Complex};

const NOTES: [&str; 12] = ["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];

pub struct Tuner {
    sample_rate: u32,
    fft_size: usize,
    planner: FftPlanner<f32>,
}

impl Tuner {
    pub fn new(sample_rate: u32) -> Self {
        let fft_size = 4096;
        let mut planner = FftPlanner::new();
        planner.plan_fft_forward(fft_size);

        Tuner {
            sample_rate,
            fft_size,
            planner,
        }
    }

    fn hann_window(index: usize, size: usize) -> f32 {
        let n = size as f32;
        let i = index as f32;
        0.5 * (1.0 - (2.0 * std::f32::consts::PI * i / (n - 1.0)).cos())
    }

    pub fn detect_frequency(&mut self, samples: &[f32]) -> Option<f32> {
        if samples.len() < self.fft_size {
            return None;
        }

        let windowed: Vec<f32> = samples[..self.fft_size]
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let window = Self::hann_window(i, self.fft_size);
                sample * window
            })
            .collect();

        let mut complex_samples: Vec<Complex<f32>> = windowed
            .iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();

        complex_samples.resize(self.fft_size, Complex::new(0.0, 0.0));

        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process(&mut complex_samples);

        let mut max_magnitude = 0.0;
        let mut max_bin = 0;

        for (i, complex) in complex_samples.iter().enumerate().take(self.fft_size / 2) {
            let magnitude = complex.norm();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                max_bin = i;
            }
        }

        if max_magnitude < 0.01 {
            return None;
        }

        let freq = (max_bin as f32 * self.sample_rate as f32) / self.fft_size as f32;

        let refined_freq = self.refine_frequency(&complex_samples, max_bin, freq);

        if refined_freq > 20.0 && refined_freq < 5000.0 {
            Some(refined_freq)
        } else {
            None
        }
    }

    fn refine_frequency(&self, fft_result: &[Complex<f32>], bin: usize, rough_freq: f32) -> f32 {
        if bin == 0 || bin >= fft_result.len() / 2 - 1 {
            return rough_freq;
        }

        let mag_prev = fft_result[bin - 1].norm();
        let mag_curr = fft_result[bin].norm();
        let mag_next = fft_result[bin + 1].norm();

        let denom = mag_prev + mag_curr + mag_next;
        if denom < 1e-10 {
            return rough_freq;
        }

        let offset = (mag_next - mag_prev) / (2.0 * denom);
        let bin_center = bin as f32 + offset;
        (bin_center * self.sample_rate as f32) / self.fft_size as f32
    }

    pub fn frequency_to_note(&self, frequency: f32, a4_freq: f32) -> (String, i32, f32) {
        let semitones_from_a4 = 12.0 * (frequency / a4_freq).log2();
        let rounded_semitones = semitones_from_a4.round() as i32;
        let octave = 4 + (rounded_semitones + 9) / 12;
        let note_index = ((rounded_semitones % 12) + 12) % 12;
        let note_name = NOTES[note_index as usize].to_string();

        let target_freq = a4_freq * 2.0_f32.powf(rounded_semitones as f32 / 12.0);
        let deviation_cents = 1200.0 * (frequency / target_freq).log2();

        (note_name, octave, deviation_cents)
    }

    pub fn note_name_to_frequency(note_name: &str, octave: i32, a4_freq: f32) -> f32 {
        let note_index = NOTES.iter().position(|&n| n == note_name).unwrap_or(0) as i32;
        let semitones_from_a4 = (octave - 4) * 12 + (note_index - 9);
        a4_freq * 2.0_f32.powf(semitones_from_a4 as f32 / 12.0)
    }
}

