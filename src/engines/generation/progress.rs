use super::evolution_engine::ProgressCallback;

pub struct ConsoleProgressCallback;

impl ProgressCallback for ConsoleProgressCallback {
    fn on_generation_start(&mut self, generation: usize) {
        println!("Generation {} starting...", generation + 1);
    }

    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hof_size: usize) {
        println!(
            "Generation {} complete. Best fitness: {:.4}, Hall of Fame size: {}",
            generation + 1, best_fitness, hof_size
        );
    }

    fn on_strategy_evaluated(&mut self, strategy_num: usize, total: usize) {
        if strategy_num % 10 == 0 || strategy_num == total {
            println!("  Evaluated {}/{} strategies", strategy_num, total);
        }
    }
}

// For IPC communication with UI
pub struct IpcProgressCallback {
    sender: std::sync::mpsc::Sender<ProgressMessage>,
}

pub enum ProgressMessage {
    GenerationStart(usize),
    GenerationComplete { generation: usize, best_fitness: f64, hof_size: usize },
    StrategyEvaluated { current: usize, total: usize },
}

impl IpcProgressCallback {
    pub fn new(sender: std::sync::mpsc::Sender<ProgressMessage>) -> Self {
        Self { sender }
    }
}

impl ProgressCallback for IpcProgressCallback {
    fn on_generation_start(&mut self, generation: usize) {
        let _ = self.sender.send(ProgressMessage::GenerationStart(generation));
    }

    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hof_size: usize) {
        let _ = self.sender.send(ProgressMessage::GenerationComplete {
            generation,
            best_fitness,
            hof_size,
        });
    }

    fn on_strategy_evaluated(&mut self, strategy_num: usize, total: usize) {
        let _ = self.sender.send(ProgressMessage::StrategyEvaluated {
            current: strategy_num,
            total,
        });
    }
}
