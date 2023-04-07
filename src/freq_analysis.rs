// src/freq_analysis.rs
use aho_corasick::AhoCorasickBuilder;
use std::cmp::min;
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

pub fn character_frequency(text: &str) -> HashMap<char, f64> {
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

pub fn index_of_coincidence(text: &str) -> f64 {
    let frequency_map = character_frequency(text);
    let text_length = text.chars().filter(|c| c.is_ascii_alphabetic()).count() as f64;
    let mut ic = 0.0;

    for (c, freq) in ENGLISH_FREQUENCIES.iter() {
        if let Some(char_count) = frequency_map.get(c) {
            ic += freq * (*char_count as f64) * (*char_count as f64 - 1.0);
        }
    }

    ic / (text_length * (text_length - 1.0))
}

fn kasiski_examination(text: &str) -> Vec<usize> {
    let mut distances: HashMap<usize, usize> = HashMap::new();
    let text_upper = text.to_ascii_uppercase();
    let text_len = text_upper.len();

    //println!("Text length: {}", text_len);

    for seq_len in 3..=5 {
        //println!("Sequence length: {}", seq_len);
        for start in 0..(text_len - seq_len) {
            if start + seq_len >= text_len {
                break;
            }
            let sequence = &text_upper[start..start + seq_len];
            //println!("Current sequence: {}", sequence);

            let patterns = vec![sequence];
            let ac = AhoCorasickBuilder::new().build(&patterns);
            let matches = ac.find_iter(&text_upper);

            let mut last_offset = None;
            for mat in matches {
                let offset = mat.start();
                if last_offset.is_some() && last_offset.unwrap() == offset {
                    continue;
                }
                last_offset = Some(offset);

                if offset > start {
                    let distance = offset - start;
                    *distances.entry(distance).or_insert(0) += 1;
                    //println!("Found repeating sequence at distance: {}", distance);
                }
            }
        }
    }

    let mut possible_key_lengths: Vec<usize> = Vec::new();
    let mut sorted_distances: Vec<(&usize, &usize)> = distances.iter().collect();
    sorted_distances.sort_by(|a, b| b.1.cmp(a.1));

    //println!("Sorted distances: {:?}", sorted_distances);

    for &(dist, _) in &sorted_distances {
        //println!("Distance: {}", dist);
        let mut divisors: Vec<usize> = Vec::new();
        for i in 1..=((*dist as f64).sqrt() as usize) {
            if dist % i == 0 {
                divisors.push(i);
                if i != dist / i {
                    divisors.push(dist / i);
                }
            }
        }
        divisors.sort_unstable();
        for divisor in divisors {
            if divisor > 1 {
                // Change this condition
                possible_key_lengths.push(divisor);
                //println!("Divisor: {}", divisor);
            }
        }
    }

    //println!("Possible key lengths: {:?}", possible_key_lengths);

    possible_key_lengths.truncate(min(possible_key_lengths.len(), 20));
    possible_key_lengths
}

pub fn analyze_text(text: &str) -> (HashMap<char, f64>, f64, Vec<usize>) {
    let frequency_map = character_frequency(text);
    let ic = index_of_coincidence(text);
    let possible_key_lengths = kasiski_examination(text);

    //println!(
    //    "Freq_map: {:?}, ic: {}, possible_key_lengths: {:?}",
    //    frequency_map, ic, possible_key_lengths
    //);
    (frequency_map, ic, possible_key_lengths)
}
