// src/k_len_estimator.rs
use crate::freq_analysis::{analyze_text, index_of_coincidence};

use std::collections::HashMap;

fn find_gcd_of_list(numbers: &[usize]) -> usize {
    fn gcd(a: usize, b: usize) -> usize {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

    numbers.iter().fold(0, |acc, &num| gcd(acc, num))
}

// Add the new strategy to the KeyLengthEstimationStrategy enum:
pub enum KeyLengthEstimationStrategy {
    Autocorrelation,
    IndexOfCoincidence,
    FriedmanTest,
}

pub fn estimate_key_length_using_multiple_strategies(
    strategies: &[KeyLengthEstimationStrategy],
    possible_key_lengths: &[usize],
    text: &str,
    specified_key_length: Option<usize>,
) -> usize {
    // If a key length is specified, return it directly without any computation
    if let Some(key_length) = specified_key_length {
        return key_length;
    }

    let mut candidates: Vec<(usize, f64)> = Vec::new();

    for &key_length in possible_key_lengths {
        let mut sum_scores = 0.0;

        for strategy in strategies {
            let score = match strategy {
                KeyLengthEstimationStrategy::Autocorrelation => {
                    autocorrelation_score(key_length, text)
                } // Add more strategies here when needed
                KeyLengthEstimationStrategy::IndexOfCoincidence => {
                    index_of_coincidence_score(key_length, text)
                }
                KeyLengthEstimationStrategy::FriedmanTest => {
                    let (score, len) = friedman_test(key_length, text);
                    score
                }
            };
            sum_scores += score;
        }

        let avg_score = sum_scores / strategies.len() as f64;
        println!("Key length: {}, Average Score: {}", key_length, avg_score);
        candidates.push((key_length, avg_score));
    }

    candidates
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
        .0
}

fn autocorrelation_score(key_length: usize, text: &str) -> f64 {
    let mut correlation_count = 0;

    for i in 0..text.len() - key_length {
        if text.chars().nth(i) == text.chars().nth(i + key_length) {
            correlation_count += 1;
        }
    }

    correlation_count as f64
}

pub fn index_of_coincidence_score(key_length: usize, text: &str) -> f64 {
    let text_length = text.len() as f64;

    let mut sum_ioc = 0.0;
    let mut count = 0;

    for i in 0..key_length {
        let mut freq_map: HashMap<char, usize> = HashMap::new();

        for (j, c) in text.chars().enumerate() {
            if j % key_length == i {
                *freq_map.entry(c).or_insert(0) += 1;
            }
        }

        let ioc: f64 = freq_map
            .values()
            .map(|&freq| freq as f64 * (freq as f64 - 1.0))
            .sum::<f64>()
            / (text_length * (text_length - 1.0));

        sum_ioc += ioc;
        count += 1;
    }

    sum_ioc / count as f64
}

fn friedman_test(max_key_length: usize, text: &str) -> (f64, usize) {
    let english_ic = 0.0667;

    let mut best_key_length = 1;
    let mut smallest_ic_difference = f64::MAX;

    for key_length in 1..=max_key_length {
        let mut sum_ic = 0.0;

        for i in 0..key_length {
            let substring: String = text
                .chars()
                .enumerate()
                .filter_map(|(j, c)| if j % key_length == i { Some(c) } else { None })
                .collect();

            let (freq_map, _, _) = analyze_text(&substring);
            let ic = calculate_index_of_coincidence(&freq_map, substring.len());

            sum_ic += ic;
        }

        let avg_ic = sum_ic / key_length as f64;
        let ic_difference = (english_ic - avg_ic).abs();

        if ic_difference < smallest_ic_difference {
            smallest_ic_difference = ic_difference;
            best_key_length = key_length;
        }
    }

    (smallest_ic_difference, best_key_length)
}

fn calculate_index_of_coincidence(freq_map: &HashMap<char, f64>, length: usize) -> f64 {
    freq_map
        .values()
        .map(|&freq| freq * (freq - 1.0))
        .sum::<f64>()
        / (length * (length - 1)) as f64
}

fn calculate_mutual_ic(freq_map1: &HashMap<char, f64>, freq_map2: &HashMap<char, f64>) -> f64 {
    freq_map1
        .iter()
        .map(|(&c, &freq1)| freq1 * freq_map2.get(&c).unwrap_or(&0.0))
        .sum()
}
