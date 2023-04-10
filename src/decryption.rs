// src/decryption.rs
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::freq_analysis::character_frequency;
use crate::freq_analysis::ENGLISH_FREQUENCIES;

fn generate_random_key(key_length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..key_length)
        .map(|_| (b'A' + rng.gen_range(0..26)) as char)
        .collect()
}

fn evaluate_key_fitness(decrypted_text: &str) -> f64 {
    let freq_map = character_frequency(&decrypted_text);
    ENGLISH_FREQUENCIES
        .iter()
        .map(|(c, expected_freq)| {
            let actual_freq = freq_map.get(c).unwrap_or(&0.0);
            (actual_freq - expected_freq).abs()
        })
        .sum::<f64>()
}

fn genetic_guess_key(text: &str, key_length: usize) -> String {
    let population_size = 10000;
    let generations = 10000;
    let crossover_rate = 0.8;
    let mutation_rate = 0.15;
    let tournament_size = 100;

    let mut rng = rand::thread_rng();

    let mut population: Vec<String> = (0..population_size)
        .map(|_| generate_random_key(key_length))
        .collect();

    for i in 0..generations {
        // Evaluate the fitness of each key
        println!("{}%", (i as f64 / generations as f64) * 100 as f64);
        let mut fitnesses: Vec<(String, f64)> = population
            .iter()
            .map(|key| {
                (
                    key.clone(),
                    evaluate_key_fitness(&vigenere_decrypt(text, key)),
                )
            })
            .collect();

        // Sort the population by fitness (ascending)
        fitnesses.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

        // Select the best performing keys for reproduction
        let mut new_population: Vec<String> = fitnesses
            .iter()
            .take(tournament_size)
            .map(|(key, _)| key.clone())
            .collect();

        // Generate offspring
        while new_population.len() < population_size {
            let parents: Vec<&String> = population.choose_multiple(&mut rng, 2).collect();

            let mut offspring = if rng.gen::<f64>() < crossover_rate {
                // Apply crossover
                let crossover_point = rng.gen_range(1..key_length);
                let child1 = format!(
                    "{}{}",
                    &parents[0][..crossover_point],
                    &parents[1][crossover_point..]
                );
                let child2 = format!(
                    "{}{}",
                    &parents[1][..crossover_point],
                    &parents[0][crossover_point..]
                );
                vec![child1, child2]
            } else {
                parents.into_iter().cloned().collect()
            };

            // Apply mutation
            for child in offspring.iter_mut() {
                for i in 0..key_length {
                    if rng.gen::<f64>() < mutation_rate {
                        let new_char = (b'A' + rng.gen_range(0..26)) as char;
                        child.replace_range(i..i + 1, &new_char.to_string());
                    }
                }
            }

            new_population.append(&mut offspring);
        }

        population = new_population;
    }

    population[0].clone()
}
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

/*
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
*/

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

            let (best_shift, _) = (0..26)
                .map(|shift| {
                    let chi_squared = chi_squared_test(&column_text, shift, &ENGLISH_FREQUENCIES);
                    (shift, chi_squared)
                })
                .min_by(|(_, chi1), (_, chi2)| chi1.partial_cmp(chi2).unwrap())
                .unwrap();

            (b'A' + best_shift as u8) as char
        })
        .collect()
}
/*
fn guess_key(text: &str, key_length: usize) -> String {
    (0..key_length)
        .map(|i| {
            // Extract every i-th character in the text
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

            // Find the best shift (cipher key) for each column
            let (best_shift, _) = (0..26)
                .map(|shift| {
                    let ic = index_of_coincidence(&column_text, shift, &ENGLISH_FREQUENCIES);
                    (shift, ic)
                })
                .max_by(|(_, ic1), (_, ic2)| ic1.partial_cmp(ic2).unwrap())
                .unwrap();

            // Convert the shift value to the corresponding character
            (b'A' + best_shift as u8) as char
        })
        .collect()
}
*/
pub fn decrypt_vigenere(
    ciphertext: &str,
    key_length: usize,
    key_option: Option<String>,
) -> (String, String) {
    if let Some(key) = key_option {
        return (key.clone(), vigenere_decrypt(ciphertext, &key));
    }
    //let key = genetic_guess_key(ciphertext, key_length);
    let key = guess_key(ciphertext, key_length);
    let decrypted_text = vigenere_decrypt(ciphertext, &key);
    (key, decrypted_text)
}
