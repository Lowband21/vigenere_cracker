// src/decryption.rs

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

fn guess_key(text: &str, key_length: usize) -> String {
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

            let freq_map = character_frequency(&column_text);
            let mut max_ic = 0.0;
            let mut best_shift = 0;
            for shift in 0..26 {
                let mut ic = 0.0;
                for (c, freq) in ENGLISH_FREQUENCIES.iter() {
                    let shifted_c = (((*c as u8 - b'A' + shift as u8) % 26) + b'A') as char;
                    if let Some(char_freq) = freq_map.get(&shifted_c) {
                        ic += char_freq * freq;
                    }
                }
                if ic > max_ic {
                    max_ic = ic;
                    best_shift = shift;
                }
            }
            (b'A' + best_shift as u8) as char
        })
        .collect()
}

pub fn decrypt_vigenere(
    ciphertext: &str,
    key_length: usize,
    key_option: Option<String>,
) -> (String, String) {
    if let Some(key) = key_option {
        return (key.clone(), vigenere_decrypt(ciphertext, &key));
    }
    let key = guess_key(ciphertext, key_length);
    let decrypted_text = vigenere_decrypt(ciphertext, &key);
    (key, decrypted_text)
}
