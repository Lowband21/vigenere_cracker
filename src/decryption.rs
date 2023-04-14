// src/decryption.rs
use crate::logger::log_debug;
use std::collections::HashMap;

pub const ENGLISH_FREQUENCIES: [(char, f64); 26] = [
    ('A', 0.08167),
    ('B', 0.01492),
    ('C', 0.02782),
    ('D', 0.04253),
    ('E', 0.12702),
    ('F', 0.02228),
    ('G', 0.02015),
    ('H', 0.06094),
    ('I', 0.06966),
    ('J', 0.00153),
    ('K', 0.00772),
    ('L', 0.04025),
    ('M', 0.02406),
    ('N', 0.06749),
    ('O', 0.07507),
    ('P', 0.01929),
    ('Q', 0.00095),
    ('R', 0.05987),
    ('S', 0.06327),
    ('T', 0.09056),
    ('U', 0.02758),
    ('V', 0.00978),
    ('W', 0.02360),
    ('X', 0.00150),
    ('Y', 0.01974),
    ('Z', 0.00074),
];

// Decrypts Vigenère ciphertext using the provided key.
fn vigenere_decrypt(ciphertext: &str, key: &str) -> String {
    ciphertext
        .chars()
        .zip(key.chars().cycle())
        .map(|(c, k)| {
            // Check if character is alphabetic, else return the character.
            if c.is_ascii_alphabetic() {
                // Determine character base (lowercase or uppercase).
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                let c = c as u8 - base;
                let k = k as u8 - base;
                // Decrypt the character using Vigenère decryption.
                let decrypted = (c + 26 - k) % 26;
                (decrypted + base) as char
            } else {
                c
            }
        })
        .collect()
}

// Computes the chi-squared test value for a given text, shift, and English frequencies.
fn chi_squared_test(text: &str, shift: usize, english_frequencies: &[(char, f64)]) -> f64 {
    let freq_map = character_frequency_f64(&text);
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

// Finds the most likely key given the text and key length.
fn guess_key(text: &str, key_length: usize) -> String {
    log_debug(format!("Computed values during key length finding:"));

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

            log_debug(format!(
                "Column {}: Best shift: {}, Chi-squared: {:.4}",
                i, best_shift, best_chi_squared
            ));

            (b'A' + best_shift as u8) as char
        })
        .collect()
}

// Decrypts Vigenère ciphertext with a given key length and optional key,
// returning the key, decrypted text, and confidence.
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

// Calculates the mutual index of coincidence (MIC) between the text and English frequencies.
fn mutual_index_of_coincidence(text: &str, english_freq: [(char, f64); 26]) -> f64 {
    let freq_map = character_frequency_f64(text);
    let total_chars = text.chars().count() as f64;

    let mut mic = 0.0;

    for (ch, e_freq) in english_freq.iter() {
        if let Some(freq) = freq_map.get(ch) {
            mic += (freq / total_chars) * (e_freq / 26.0);
        }
    }
    log_debug(format!("MIC: {}", mic));

    mic
}

// Calculates the confidence level of the decryption based on MIC and average English MIC.
fn confidence_level(mic: f64, english_mic: f64) -> f64 {
    (mic / english_mic) * 100.0
}

// Computes the frequency of characters in the given text as a HashMap with f64 values.
pub fn character_frequency_f64(text: &str) -> HashMap<char, f64> {
    let mut frequency_map = HashMap::new();
    let mut char_count = 0;

    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            let uppercase_c = c.to_ascii_uppercase();
            *frequency_map.entry(uppercase_c).or_insert(0.0) += 1.0;
            char_count += 1;
        }
    }

    for (_, freq) in frequency_map.iter_mut() {
        *freq /= char_count as f64;
    }

    frequency_map
}
