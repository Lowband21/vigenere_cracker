// src/genetic_decryption.rs

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
