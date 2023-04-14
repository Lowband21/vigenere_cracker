// src/k_len_estimator.rs
use crate::logger::log_debug;
use itertools::Itertools;
use std::collections::HashMap;

fn find_gcd_of_list(numbers: Vec<usize>) -> usize {
    fn gcd(a: usize, b: usize) -> usize {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

    numbers
        .iter()
        .filter(|x| x > &&(5 as usize))
        .fold(0, |acc, &num| gcd(acc, num))
}

#[derive(PartialEq)]
pub enum KeyLengthEstimationStrategy {
    Autocorrelation,
    //IndexOfCoincidence,
    //FriedmanTest,
    GCD,
}

pub fn estimate_key_length_using_multiple_strategies(
    strategies: &[KeyLengthEstimationStrategy],
    mut possible_key_lengths: Vec<usize>,
    text: &str,
    specified_key_length: Option<usize>,
    frequency_multiplier: f64,
) -> usize {
    // If a key length is specified, return it directly without any computation
    if let Some(key_length) = specified_key_length {
        return key_length;
    }

    //let friedman_lens: &mut Vec<usize> = &mut Vec::new();
    let mut len = 0;
    let mut candidates: Vec<(usize, f64)> = Vec::new();

    // Score each possible key
    for key_length in possible_key_lengths.clone() {
        let mut sum_scores = 0.0;

        for strategy in strategies {
            let _ = match strategy {
                KeyLengthEstimationStrategy::Autocorrelation => {
                    let score = autocorrelation_score(key_length, text);
                    log_debug(format!("Autocorrelation Score: {}", score));
                    sum_scores += score;
                }
                /*
                KeyLengthEstimationStrategy::IndexOfCoincidence => {
                    let score = index_of_coincidence_score(key_length, text);
                    log_debug(format!("Index of Coincidence Score: {}", score));
                    sum_scores += score
                }
                KeyLengthEstimationStrategy::FriedmanTest => {
                    let (score, len) = friedman_test(key_length, text);
                    friedman_lens.push(len.clone());
                    log_debug(format!("Friedman Test Score: {}, Len: {}", score, len));
                    sum_scores += score * 10.0
                }*/
                KeyLengthEstimationStrategy::GCD => {
                    len = find_gcd_of_list(possible_key_lengths.clone());
                }
            };
        }
        //for i in &mut *friedman_lens {
        //    possible_key_lengths.push(*i);
        //}
        // Weight towards the result of find_gcd_of_list
        if strategies.contains(&KeyLengthEstimationStrategy::GCD) && len > 5 {
            possible_key_lengths.push(len);
        }

        let frequency_map = create_frequency_map(&possible_key_lengths.clone());

        log_debug(format!(
            "Possible key lengths after GCD: {:?}",
            possible_key_lengths
        ));

        // Weight towards lengths that appear multiple times
        let mut frequency_bonus = 0.0;
        if key_length > 3 {
            frequency_bonus =
                (*frequency_map.get(&key_length).unwrap_or(&0)) as f64 * frequency_multiplier;
        }

        // Calculate avg score
        let avg_score = (sum_scores / strategies.len() as f64) + frequency_bonus;

        log_debug(format!(
            "Key length: {}, Average Score: {}",
            key_length, avg_score
        ));
        candidates.push((key_length, avg_score));
    }

    // Print the top 5 candidates
    log_debug(format!("Top Candidates: "));
    let top_candidates: Vec<_> = candidates
        .iter()
        .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap()) // Sort candidates in descending order of score
        .take(5)
        .collect();

    for &(len, score) in &top_candidates {
        log_debug(format!("({}, {}), ", len, score));
    }
    candidates
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
        .0
}

// Helper for converting from Vec to HashMap
pub fn create_frequency_map(key_lengths: &Vec<usize>) -> HashMap<usize, usize> {
    let mut frequency_map = HashMap::new();

    for key_length in key_lengths {
        *frequency_map.entry(*key_length).or_insert(0) += 1;
    }

    frequency_map
}

pub fn autocorrelation_score(key_length: usize, text: &str) -> f64 {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut correlation_count = 0;

    for i in 0..len - key_length {
        if bytes[i] == bytes[i + key_length] {
            correlation_count += 1;
        }
    }

    // Normalize
    let max_possible_matches = len - key_length;
    let normalized_score = (correlation_count as f64) / (max_possible_matches as f64);
    1.0 + normalized_score // normalize to the range [0.5, 1.5]
}

/*
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

            //let (freq_map, _, _) = analyze_text(&substring);
            let ic = calculate_index_of_coincidence(&substring, substring.len());

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

fn calculate_index_of_coincidence(text: &str, length: usize) -> f64 {
    fn create_frequency_map(text: &str) -> HashMap<char, usize> {
        let mut freq_map = HashMap::new();

        for c in text.chars() {
            *freq_map.entry(c).or_insert(0) += 1;
        }

        freq_map
    }
    let freq_map = create_frequency_map(text);

    freq_map
        .values()
        .map(|&count| count * (count - 1))
        .sum::<usize>() as f64
        / (length * (length - 1)) as f64
}
*/
