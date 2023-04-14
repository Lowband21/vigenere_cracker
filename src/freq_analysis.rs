// src/freq_analysis.rs
use crate::logger::log_timing;
use aho_corasick::AhoCorasickBuilder;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::Instant;

lazy_static! {
    pub static ref ENGLISH_FREQUENCIES: HashMap<char, f64> = [
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
    ]
    .iter()
    .cloned()
    .collect();
}

// Computes the frequency of ASCII alphabetic characters in the given text.
pub fn character_frequency_usize(text: &str) -> Vec<usize> {
    // Initialize a frequency map with 26 entries for each English letter.
    let mut frequency_map = vec![0; 26];

    // Iterate through the characters of the input text.
    for c in text.chars() {
        // If the character is an ASCII alphabetic character,
        if c.is_ascii_alphabetic() {
            // Compute the index corresponding to the lowercase character
            let index = (c.to_ascii_lowercase() as usize) - 'a' as usize;
            // Increment the frequency count for that character.
            frequency_map[index] += 1;
        }
    }

    frequency_map
}

// Computes the Index of Coincidence (IC) for the given text.
pub fn index_of_coincidence(text: &str) -> f64 {
    // Obtain the frequency map of the characters in the text.
    let frequency_map = character_frequency_usize(text);
    // Count the number of ASCII alphabetic characters in the text.
    let text_length = text.chars().filter(|c| c.is_ascii_alphabetic()).count() as f64;
    let mut ic = 0.0;

    // Calculate the IC value using the character frequencies.
    for &freq in frequency_map.iter() {
        ic += (freq as f64) * (freq as f64 - 1.0);
    }

    // Normalize the IC value by the text length.
    ic / (text_length * (text_length - 1.0))
}

// Finds the divisors of the given number.
fn find_divisors(number: usize) -> Vec<usize> {
    let mut divisors = Vec::new();
    // Iterate from 1 to the square root of the number.
    for i in 1..=((number as f64).sqrt() as usize) {
        // If the number is divisible by i,
        if number % i == 0 {
            // Add i to the list of divisors.
            divisors.push(i);
            // If i is not the square root of the number, add the other divisor.
            if i != number / i {
                divisors.push(number / i);
            }
        }
    }
    // Sort the divisors in ascending order.
    divisors.sort_unstable();
    divisors
}

// Performs the Kasiski examination on the given text.
fn kasiski_examination(text: &str) -> Vec<usize> {
    // Initialize a hashmap to store distances between repeated sequences.
    let mut distances: HashMap<usize, usize> = HashMap::new();
    // Count the characters in the text.
    let text_len = text.chars().count();

    // Set the starting sequence length based on the text length.
    let mut start = 3;
    if text_len < 100 {
        start = 2;
    }

    // Iterate through sequence lengths from the starting length to 4.
    for seq_len in start..=4 {
        // Iterate through the starting positions of the sequences.
        for start in 0..(text_len - seq_len) {
            // Extract the sequence from the text.
            let sequence: &str = &text[start..start + seq_len];
            // Create the Aho-Corasick automaton for the sequence.
            let patterns = vec![sequence];
            let ac = AhoCorasickBuilder::new()
                .auto_configure(&patterns)
                .build(patterns);
            // Find matches of the sequence in the text.
            let matches = ac.find_iter(&text);

            let mut last_offset = None;
            // Iterate through the matches found in the text.
            for mat in matches {
                let offset = mat.start();
                // Skip the match if it is the same as the last offset.
                if last_offset.is_some() && last_offset.unwrap() == offset {
                    continue;
                }
                last_offset = Some(offset);

                // If the offset is greater than the starting position,
                if offset > start {
                    // Calculate the distance between the current and previous occurrences.
                    let distance = offset - start;
                    // Increment the count for this distance in the hashmap.
                    *distances.entry(distance).or_insert(0) += 1;
                }
            }
        }
    }

    // Initialize a vector to store the possible key lengths.
    let mut possible_key_lengths: Vec<usize> = Vec::new();
    // Collect the distances from the hashmap and sort them by their counts in descending order.
    let mut sorted_distances: Vec<(&usize, &usize)> = distances.iter().collect();
    sorted_distances.sort_by(|a, b| b.1.cmp(a.1));

    // Iterate through the sorted distances.
    for &(dist, _) in &sorted_distances {
        // Find the divisors of the distance.
        let divisors = find_divisors(*dist);
        // Iterate through the divisors.
        for divisor in divisors {
            // If the divisor is greater than 1, add it to the list of possible key lengths.
            if divisor > 1 {
                possible_key_lengths.push(divisor);
            }
        }
    }

    // Truncate the list of possible key lengths to a maximum of 15 elements.
    possible_key_lengths.truncate(std::cmp::min(possible_key_lengths.len(), 15));
    possible_key_lengths
}

pub fn analyze_text(text: &str) -> (f64, Vec<usize>) {
    let start = Instant::now();
    let ic = index_of_coincidence(text);
    let ic_dur = start.elapsed();
    let start = Instant::now();
    let possible_key_lengths = kasiski_examination(text);
    let kasiski_dur = start.elapsed();

    log_timing(format!(
        "IC time: {} seconds and {} milliseconds",
        ic_dur.as_secs(),
        ic_dur.subsec_millis()
    ));
    log_timing(format!(
        "Kasiski decryption time: {} seconds and {} milliseconds",
        kasiski_dur.as_secs(),
        kasiski_dur.subsec_millis()
    ));

    (ic, possible_key_lengths)
}
