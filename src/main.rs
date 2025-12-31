// main.rs

use biosimrust::simulator::Simulator;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let argc = args.len();
    
    let mut simulator = Simulator::new();
    simulator.simulator(argc, args);
}

