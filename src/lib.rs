// src/lib.rs
pub mod freq_analysis;
use freq_analysis::character_frequency;

pub const LOG_LEVEL: LogState = LogState::WARN;

#[derive(Debug)]
pub enum LogState {
    WARN,
    ERROR,
    DEBUG,
}

pub fn log(message: String) {
    println!("{:?}: \"{}\"", LOG_LEVEL, message)
}

fn index_of_coincidence(text: &str, shift: usize, english_frequencies: &[(char, f64)]) -> f64 {
    let freq_map = character_frequency(&text);

    let mut ic = 0.0;
    for (c, freq) in english_frequencies.iter() {
        let shifted_c = (((*c as u8 - b'A' + shift as u8) % 26) + b'A') as char;
        if let Some(char_freq) = freq_map.get(&shifted_c) {
            ic += char_freq * freq;
        }
    }

    ic
}
