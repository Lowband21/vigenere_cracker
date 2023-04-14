// src/main.rs

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use requestty::{Answer, Question};

mod decryption;
use decryption::decrypt_vigenere;
mod freq_analysis;
use freq_analysis::analyze_text;
mod k_len_estimator;
use k_len_estimator::{estimate_key_length_using_multiple_strategies, KeyLengthEstimationStrategy};
mod logger;
use logger::{log_debug, log_info, log_timing};

// Read text from file
fn read_ciphertext(file_path: &Path) -> Result<String, io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut ciphertext = String::new();
    for line in reader.lines() {
        ciphertext.push_str(&line?);
    }
    Ok(ciphertext)
}

// Main
fn main() {
    let input_path = Path::new("./input");
    // Collect files at path
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

    // Selection prompt
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

    // Begin timing
    let start_time = Instant::now();
    let mut results = Vec::new();

    // Process each file
    for selected_file in selected_files {
        let ciphertext_file = input_path.join(&selected_file);
        let ciphertext = match read_ciphertext(&ciphertext_file) {
            Ok(content) => content,
            Err(e) => {
                log_debug(format!(
                    "Error reading file {}: {}",
                    ciphertext_file.display(),
                    e
                ));
                std::process::exit(1);
            }
        };
        // Run decryption
        let result = run(ciphertext);
        results.push((selected_file, result));
    }

    // Summarize results
    log_info(format!("\nSummary:"));
    for (file, (decrypted_text, elapsed, ic, key_length, key, confidence)) in results {
        log_info(format!("File: {}", file));
        log_timing(format!(
            "Decryption took {} seconds and {} milliseconds",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        ));
        log_info(format!("Index of Coincidence: {:.6}", ic));
        log_info(format!("Estimated key length: {}", key_length));
        log_info(format!("Decrypted key: {}", key));
        log_info(format!(
            "Decrypted text with confidence {:.2}%: {}",
            100.0 - (confidence * 1000.0),
            decrypted_text
        ));
    }
    let total_elapsed = start_time.elapsed();
    // Print the duration in a human-readable format
    log_timing(format!(
        "All decryption(s) took {} seconds and {} milliseconds",
        total_elapsed.as_secs(),
        total_elapsed.subsec_millis()
    ));
}

fn run(ciphertext: String) -> (String, Duration, f64, usize, String, f64) {
    let mut summary = Vec::new();

    // Time and run text analysis
    let start_time = Instant::now();
    let (ic, possible_key_lengths) = analyze_text(&ciphertext);
    let analyze_text_duration = start_time.elapsed();
    summary.push(("Analyze text", analyze_text_duration));

    // Time and run key length estimation
    let start_time = Instant::now();
    let key_length = estimate_key_length_using_multiple_strategies(
        &[
            KeyLengthEstimationStrategy::Autocorrelation,
            KeyLengthEstimationStrategy::GCD,
        ],
        possible_key_lengths.clone(),
        &ciphertext,
        None,
        5.0,
    );
    let estimate_key_length_duration = start_time.elapsed();
    summary.push(("Estimate key length", estimate_key_length_duration));

    // Time and run decryption
    let start_time = Instant::now();
    let (key, decrypted_text, confidence) =
        decrypt_vigenere(&ciphertext.to_uppercase(), key_length, None);
    let decrypt_vigenere_duration = start_time.elapsed();
    summary.push(("Decrypt Vigenere", decrypt_vigenere_duration));

    // Calculate total duration
    let total_duration =
        analyze_text_duration + estimate_key_length_duration + decrypt_vigenere_duration;

    // Print Summary
    for (name, duration) in summary {
        log_timing(format!(
            "{} took {} seconds and {} milliseconds",
            name,
            duration.as_secs(),
            duration.subsec_millis()
        ));
    }

    log_timing(format!(
        "Total decryption time: {} seconds and {} milliseconds",
        total_duration.as_secs(),
        total_duration.subsec_millis()
    ));

    (
        decrypted_text,
        total_duration,
        ic,
        key_length,
        key,
        confidence,
    )
}
