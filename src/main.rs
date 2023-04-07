// src/main.rs

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

mod decryption;
use decryption::decrypt_vigenere;
mod freq_analysis;
use freq_analysis::analyze_text;
mod k_len_estimator;
use k_len_estimator::{estimate_key_length_using_multiple_strategies, KeyLengthEstimationStrategy};

fn read_ciphertext(file_path: &Path) -> Result<String, io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut ciphertext = String::new();
    for line in reader.lines() {
        ciphertext.push_str(&line?);
    }
    Ok(ciphertext)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut ciphertext_file = Path::new("./input/ciphertext2.txt");
    if args.len() != 2 {
        eprintln!("Usage: {} <ciphertext_file>", args[0]);
        //std::process::exit(1);
    } else {
        ciphertext_file = Path::new(&args[1]);
    }

    let ciphertext = match read_ciphertext(ciphertext_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", args[0], e);
            std::process::exit(1);
        }
    };

    let start_time = Instant::now();
    let (_, ic, possible_key_lengths) = analyze_text(&ciphertext);
    let key_length = estimate_key_length_using_multiple_strategies(
        &[
            KeyLengthEstimationStrategy::Autocorrelation,
            //KeyLengthEstimationStrategy::IndexOfCoincidence,
            //'KeyLengthEstimationStrategy::FriedmanTest,
        ],
        &possible_key_lengths,
        &ciphertext,
        None,
    );
    let (key, decrypted_text) = decrypt_vigenere(&ciphertext.to_uppercase(), key_length, None);
    let elapsed = start_time.elapsed();
    // Print the duration in a human-readable format
    println!(
        "Decryption took {} seconds and {} milliseconds",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );

    println!("Index of Coincidence: {:.6}", ic);
    println!("Possible key lengths: {:?}", possible_key_lengths);
    println!("Estimated key length: {}", key_length);
    println!("Decrypted key: {}", key);
    println!("Decrypted text: {}", decrypted_text);
}
