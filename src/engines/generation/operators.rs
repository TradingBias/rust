use crate::engines::generation::genome::Genome;
use rand::Rng;

/// Tournament selection: pick best of K random candidates
pub fn tournament_selection<R: Rng>(
    population: &[(Genome, f64)],
    tournament_size: usize,
    rng: &mut R,
) -> Genome {
    let mut best_idx = rng.gen_range(0..population.len());
    let mut best_fitness = population[best_idx].1;

    for _ in 1..tournament_size {
        let idx = rng.gen_range(0..population.len());
        if population[idx].1 > best_fitness {
            best_idx = idx;
            best_fitness = population[idx].1;
        }
    }

    population[best_idx].0.clone()
}

/// Roulette wheel selection: probability proportional to fitness
pub fn roulette_selection<R: Rng>(
    population: &[(Genome, f64)],
    rng: &mut R,
) -> Genome {
    // Normalize fitness to probabilities
    let total_fitness: f64 = population.iter().map(|(_, f)| f.max(0.0)).sum();

    if total_fitness <= 0.0 {
        // All negative fitness, pick random
        return population[rng.gen_range(0..population.len())].0.clone();
    }

    let mut spin = rng.gen::<f64>() * total_fitness;

    for (genome, fitness) in population {
        spin -= fitness.max(0.0);
        if spin <= 0.0 {
            return genome.clone();
        }
    }

    // Fallback
    population[population.len() - 1].0.clone()
}

/// Single-point crossover: swap genome segments
pub fn crossover<R: Rng>(
    parent1: &Genome,
    parent2: &Genome,
    rng: &mut R,
) -> (Genome, Genome) {
    let len = parent1.len().min(parent2.len());
    if len <= 1 {
        return (parent1.clone(), parent2.clone());
    }

    let point = rng.gen_range(1..len);

    let mut child1 = parent1.clone();
    let mut child2 = parent2.clone();

    child1[point..].copy_from_slice(&parent2[point..]);
    child2[point..].copy_from_slice(&parent1[point..]);

    (child1, child2)
}

/// Mutation: randomly modify genes
pub fn mutate<R: Rng>(
    genome: &mut Genome,
    mutation_rate: f64,
    gene_range: std::ops::Range<u32>,
    rng: &mut R,
) {
    for gene in genome.iter_mut() {
        if rng.gen::<f64>() < mutation_rate {
            *gene = rng.gen_range(gene_range.clone());
        }
    }
}

/// Generate random genome
pub fn random_genome<R: Rng>(
    length: usize,
    gene_range: std::ops::Range<u32>,
    rng: &mut R,
) -> Genome {
    (0..length)
        .map(|_| rng.gen_range(gene_range.clone()))
        .collect()
}
