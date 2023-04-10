// src/decryption.rs
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::freq_analysis::character_frequency;
use crate::freq_analysis::ENGLISH_FREQUENCIES;

fn vigenere_decrypt(ciphertext: &str, key: &str) -> String {
    ciphertext
        .chars()
        .zip(key.chars().cycle())
        .map(|(c, k)| {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                let c = c as u8 - base;
                let k = k as u8 - base;
                let decrypted = (c + 26 - k) % 26;
                (decrypted + base) as char
            } else {
                c
            }
        })
        .collect()
}

fn chi_squared_test(text: &str, shift: usize, english_frequencies: &[(char, f64)]) -> f64 {
    let freq_map = character_frequency(&text);
    let total_chars = text.chars().count() as f64;

    let mut chi_squared = 0.0;
    for (c, expected_freq) in english_frequencies.iter() {
        let shifted_c = (((*c as u8 - b'A' + shift as u8) % 26) + b'A') as char;
        let observed_freq = freq_map.get(&shifted_c).unwrap_or(&0.0) * total_chars;
        chi_squared +=
            (observed_freq - expected_freq * total_chars).powi(2) / (expected_freq * total_chars);
    }

    chi_squared
}

pub fn create_frequency_map(key_lengths: &Vec<usize>) -> HashMap<usize, usize> {
    let mut frequency_map = HashMap::new();

    for key_length in key_lengths {
        *frequency_map.entry(*key_length).or_insert(0) += 1;
    }

    frequency_map
}

fn guess_key(text: &str, key_length: usize) -> String {
    println!("Computed values during key length finding:");

    (0..key_length)
        .map(|i| {
            let column_text: String = text
                .chars()
                .enumerate()
                .filter_map(|(idx, ch)| {
                    if idx % key_length == i {
                        Some(ch)
                    } else {
                        None
                    }
                })
                .collect();

            let (best_shift, best_chi_squared) = (0..26)
                .map(|shift| {
                    let chi_squared = chi_squared_test(&column_text, shift, &ENGLISH_FREQUENCIES);
                    (shift, chi_squared)
                })
                .min_by(|(_, chi1), (_, chi2)| chi1.partial_cmp(chi2).unwrap())
                .unwrap();

            println!(
                "Column {}: Best shift: {}, Chi-squared: {:.4}",
                i, best_shift, best_chi_squared
            );

            (b'A' + best_shift as u8) as char
        })
        .collect()
}

pub fn decrypt_vigenere(
    ciphertext: &str,
    key_length: usize,
    key_option: Option<String>,
) -> (String, String, f64) {
    if let Some(key) = key_option {
        let decrypted_text = vigenere_decrypt(ciphertext, &key);
        let mic = mutual_index_of_coincidence(&decrypted_text, ENGLISH_FREQUENCIES);
        let confidence = confidence_level(mic, 0.066); // The average MIC for English text is 0.066
        return (key.clone(), decrypted_text, confidence);
    }

    let key = guess_key(ciphertext, key_length);
    let decrypted_text = vigenere_decrypt(ciphertext, &key);
    let mic = mutual_index_of_coincidence(&decrypted_text, ENGLISH_FREQUENCIES);
    let confidence = confidence_level(mic, 0.066);

    (key, decrypted_text, confidence)
}

fn mutual_index_of_coincidence(text: &str, english_freq: [(char, f64); 26]) -> f64 {
    let freq_map = character_frequency(text);
    let total_chars = text.chars().count() as f64;

    let mut mic = 0.0;

    for (ch, e_freq) in english_freq.iter() {
        if let Some(freq) = freq_map.get(ch) {
            mic += (freq / total_chars) * (e_freq / 26.0);
        }
    }
    println!("MIC: {}", mic);

    mic
}

fn confidence_level(mic: f64, english_mic: f64) -> f64 {
    (mic / english_mic) * 100.0
}
