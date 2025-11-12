/// Deterministically consumes genes from a genome
pub struct GeneConsumer<'a> {
    genome: &'a [u32],
    position: usize,
}

impl<'a> GeneConsumer<'a> {
    pub fn new(genome: &'a [u32]) -> Self {
        Self { genome, position: 0 }
    }

    /// Consume next gene and return value
    pub fn consume(&mut self) -> u32 {
        if self.position >= self.genome.len() {
            // Wrap around if genome exhausted
            self.position = 0;
        }

        let gene = self.genome[self.position];
        self.position += 1;
        gene
    }

    /// Consume gene and map to choice index
    pub fn choose(&mut self, num_choices: usize) -> usize {
        if num_choices == 0 {
            return 0;
        }
        (self.consume() as usize) % num_choices
    }

    /// Consume gene and map to integer range
    pub fn int_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u32;
        min + (self.consume() % range) as i32
    }

    /// Consume gene and map to float range
    pub fn float_range(&mut self, min: f64, max: f64) -> f64 {
        if min >= max {
            return min;
        }
        let gene = self.consume();
        let normalized = (gene as f64) / (u32::MAX as f64); // 0.0 to 1.0
        min + normalized * (max - min)
    }

    /// Check if genes remaining
    pub fn has_genes(&self) -> bool {
        self.position < self.genome.len()
    }

    pub fn position(&self) -> usize {
        self.position
    }
}
