// src/main.rs

use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::iter::repeat_with;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use requestty::{Answer, Question};
use std::rc::Rc;

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

// Modify the main function
fn main() {
    let input_path = Path::new("./input");
    let input_entries: Vec<_> = fs::read_dir(&input_path)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                Some(entry.file_name().into_string().unwrap())
            } else {
                None
            }
        })
        .collect();

    let question = Question::multi_select("selected_files")
        .message("Select one or more ciphertext files")
        .choices(input_entries.iter())
        .build();

    let answer = &requestty::prompt_one(question).unwrap();
    let selected_files = match answer {
        Answer::ListItems(items) => items
            .into_iter()
            .map(|item| item.clone().text)
            .collect::<Vec<_>>(),
        _ => {
            eprintln!("Error selecting files");
            std::process::exit(1);
        }
    };

    let mut results = Vec::new();
    for selected_file in selected_files {
        let ciphertext_file = input_path.join(&selected_file);
        let ciphertext = match read_ciphertext(&ciphertext_file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file {}: {}", ciphertext_file.display(), e);
                std::process::exit(1);
            }
        };
        let result = run(ciphertext);
        results.push((selected_file, result));
    }

    println!("\nSummary:");
    for (file, (decrypted_text, elapsed, ic, key_length, key, confidence)) in results {
        println!("File: {}", file);
        println!(
            "Decryption took {} seconds and {} milliseconds",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
        println!("Index of Coincidence: {:.6}", ic);
        println!("Estimated key length: {}", key_length);
        println!("Decrypted key: {}", key);
        println!(
            "Decrypted text with confidence %{}: {}",
            confidence, decrypted_text
        );
    }
}

fn run(ciphertext: String) -> (String, Duration, f64, usize, String, f64) {
    let start_time = Instant::now();
    let (_, ic, possible_key_lengths) = analyze_text(&ciphertext);
    let key_length = estimate_key_length_using_multiple_strategies(
        &[
            KeyLengthEstimationStrategy::Autocorrelation,
            KeyLengthEstimationStrategy::GCD,
            //KeyLengthEstimationStrategy::IndexOfCoincidence,
            //KeyLengthEstimationStrategy::FriedmanTest,
        ],
        possible_key_lengths.clone(),
        &ciphertext,
        None,
        5.0,
    );
    let (key, decrypted_text, confidence) =
        decrypt_vigenere(&ciphertext.to_uppercase(), key_length, None);
    let elapsed = start_time.elapsed();
    // Print the duration in a human-readable format
    println!(
        "Decryption took {} seconds and {} milliseconds",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );

    (decrypted_text, elapsed, ic, key_length, key, confidence)
}
