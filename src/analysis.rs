// analysis.rs -- various reports

use std::fs::OpenOptions;
use std::io::Write;
use crate::sensors_actions::{Sensor, Action, NUM_SENSES, NUM_ACTIONS};
use crate::indiv::Indiv;
use crate::genome_neurons::{SENSOR, ACTION};
use crate::params::Params;
use crate::peeps::Peeps;
use crate::signals::Signals;
use crate::random::random_uint_range;
use crate::genome_compare::genetic_diversity;
use crate::speciation::calculate_species_statistics;
use std::sync::Mutex;

// This converts sensor numbers to descriptive strings.
pub fn sensor_name(sensor: Sensor) -> &'static str {
    match sensor {
        Sensor::Age => "age",
        Sensor::BoundaryDist => "boundary dist",
        Sensor::BoundaryDistX => "boundary dist X",
        Sensor::BoundaryDistY => "boundary dist Y",
        Sensor::LastMoveDirX => "last move dir X",
        Sensor::LastMoveDirY => "last move dir Y",
        Sensor::LocX => "loc X",
        Sensor::LocY => "loc Y",
        Sensor::LongprobePopFwd => "long probe population fwd",
        Sensor::LongprobeBarFwd => "long probe barrier fwd",
        Sensor::BarrierFwd => "short probe barrier fwd-rev",
        Sensor::BarrierLr => "short probe barrier left-right",
        Sensor::Osc1 => "osc1",
        Sensor::Population => "population",
        Sensor::PopulationFwd => "population fwd",
        Sensor::PopulationLr => "population LR",
        Sensor::Random => "random",
        Sensor::Signal0 => "signal 0",
        Sensor::Signal0Fwd => "signal 0 fwd",
        Sensor::Signal0Lr => "signal 0 LR",
        Sensor::GeneticSimFwd => "genetic similarity fwd",
        Sensor::NumSenses => "num senses",
    }
}

// Converts action numbers to descriptive strings.
pub fn action_name(action: Action) -> &'static str {
    match action {
        Action::MoveEast => "move east",
        Action::MoveWest => "move west",
        Action::MoveNorth => "move north",
        Action::MoveSouth => "move south",
        Action::MoveForward => "move fwd",
        Action::MoveX => "move X",
        Action::MoveY => "move Y",
        Action::SetResponsiveness => "set inv-responsiveness",
        Action::SetOscillatorPeriod => "set osc1",
        Action::EmitSignal0 => "emit signal 0",
        Action::KillForward => "kill fwd",
        Action::MoveReverse => "move reverse",
        Action::MoveLeft => "move left",
        Action::MoveRight => "move right",
        Action::MoveRl => "move R-L",
        Action::MoveRandom => "move random",
        Action::SetLongprobeDist => "set longprobe dist",
        Action::NumActions => "num actions",
    }
}

// This converts sensor numbers to mnemonic strings.
// Useful for later processing by graph-nnet.py.
pub fn sensor_short_name(sensor: Sensor) -> &'static str {
    match sensor {
        Sensor::Age => "Age",
        Sensor::BoundaryDist => "ED",
        Sensor::BoundaryDistX => "EDx",
        Sensor::BoundaryDistY => "EDy",
        Sensor::LastMoveDirX => "LMx",
        Sensor::LastMoveDirY => "LMy",
        Sensor::LocX => "Lx",
        Sensor::LocY => "Ly",
        Sensor::LongprobePopFwd => "LPf",
        Sensor::LongprobeBarFwd => "LPb",
        Sensor::BarrierFwd => "Bfd",
        Sensor::BarrierLr => "Blr",
        Sensor::Osc1 => "Osc",
        Sensor::Population => "Pop",
        Sensor::PopulationFwd => "Pfd",
        Sensor::PopulationLr => "Plr",
        Sensor::Random => "Rnd",
        Sensor::Signal0 => "Sg",
        Sensor::Signal0Fwd => "Sfd",
        Sensor::Signal0Lr => "Slr",
        Sensor::GeneticSimFwd => "Gen",
        Sensor::NumSenses => "NS",
    }
}

// Converts action numbers to mnemonic strings.
// Useful for later processing by graph-nnet.py.
pub fn action_short_name(action: Action) -> &'static str {
    match action {
        Action::MoveEast => "MvE",
        Action::MoveWest => "MvW",
        Action::MoveNorth => "MvN",
        Action::MoveSouth => "MvS",
        Action::MoveX => "MvX",
        Action::MoveY => "MvY",
        Action::MoveForward => "Mfd",
        Action::SetResponsiveness => "Res",
        Action::SetOscillatorPeriod => "OSC",
        Action::EmitSignal0 => "SG",
        Action::KillForward => "Klf",
        Action::MoveReverse => "Mrv",
        Action::MoveLeft => "MvL",
        Action::MoveRight => "MvR",
        Action::MoveRl => "MRL",
        Action::MoveRandom => "Mrn",
        Action::SetLongprobeDist => "LPD",
        Action::NumActions => "NA",
    }
}

// List the names of the active sensors and actions to stdout.
// "Active" means those sensors and actions that are compiled into
// the code. See sensors-actions.h for how to define the enums.
pub fn print_sensors_actions() {
    println!("Sensors:");
    for i in 0..NUM_SENSES {
        let sensor: Sensor = unsafe { std::mem::transmute(i as u8) };
        println!("  {}", sensor_name(sensor));
    }
    println!("Actions:");
    for i in 0..NUM_ACTIONS {
        let action: Action = unsafe { std::mem::transmute(i as u8) };
        println!("  {}", action_name(action));
    }
    println!();
}

// Format: 32-bit hex strings, one per gene
pub fn print_genome(indiv: &Indiv) {
    const GENES_PER_LINE: usize = 8;
    let mut count = 0;
    
    for gene in &indiv.genome {
        if count == GENES_PER_LINE {
            println!();
            count = 0;
        } else if count != 0 {
            print!(" ");
        }

        // Convert gene to u32 for hex printing
        // Gene is 6 bytes, but we'll print the first 4 bytes as u32
        let gene_bytes: [u8; 6] = unsafe { std::mem::transmute(*gene) };
        let n = u32::from_le_bytes([gene_bytes[0], gene_bytes[1], gene_bytes[2], gene_bytes[3]]);
        print!("{:08x}", n);
        count += 1;
    }
    println!();
}

// This prints a neural net in a form that can be processed with
// graph-nnet.py to produce a graphic illustration of the net.
pub fn print_igraph_edge_list(indiv: &Indiv) {
    for conn in &indiv.nnet.connections {
        // Print source
        if conn.source_type == SENSOR {
            let sensor: Sensor = unsafe { std::mem::transmute(conn.source_num) };
            print!("{}", sensor_short_name(sensor));
        } else {
            print!("N{}", conn.source_num);
        }

        print!(" ");

        // Print sink
        if conn.sink_type == ACTION {
            let action: Action = unsafe { std::mem::transmute(conn.sink_num) };
            print!("{}", action_short_name(action));
        } else {
            print!("N{}", conn.sink_num);
        }

        // Phase 4: Connection now uses f32 weight directly
        println!(" {}", conn.weight);
    }
}

// Calculate average genome length by sampling
pub fn average_genome_length(peeps: &Peeps, params: &Params) -> f32 {
    let mut count = 100u32;
    let mut number_samples = 0u32;
    let mut sum = 0u64;

    while count > 0 {
        let index = random_uint_range(1, params.population);
        if let Some(indiv) = peeps.get(index as u16) {
            sum += indiv.genome.len() as u64;
            number_samples += 1;
        }
        count -= 1;
    }

    if number_samples == 0 {
        return 0.0;
    }

    sum as f32 / number_samples as f32
}

// Track previous generation's species count for calculating new/extinct species
static PREVIOUS_SPECIES_COUNT: Mutex<u32> = Mutex::new(0);

// The epoch log contains one line per generation in a format that can be
// fed to graphlog.gp to produce a chart of the simulation progress.
// Format: generation survivors diversity avg_genome_length murder_count [species_stats...]
// When speciation is enabled, adds 8 more columns:
//   num_species avg_species_size largest_species smallest_species 
//   new_species extinct_species avg_species_age avg_stagnation
pub fn append_epoch_log(
    generation: u32,
    number_survivors: u32,
    murder_count: u32,
    peeps: &Peeps,
    params: &Params,
) {
    let log_path = format!("{}/epoch-log.txt", params.log_dir);

    if generation == 0 {
        // Create/truncate file
        if let Ok(mut file) = std::fs::File::create(&log_path) {
            let _ = file.write_all(b"");
        }
        // Reset previous species count
        let mut prev_count = PREVIOUS_SPECIES_COUNT.lock().unwrap();
        *prev_count = 0;
    }

    // Append to file
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let diversity = genetic_diversity(peeps, params);
        let avg_length = average_genome_length(peeps, params);
        
        // Get previous species count for calculating new/extinct species
        let previous_species_count = {
            let prev_count = PREVIOUS_SPECIES_COUNT.lock().unwrap();
            *prev_count
        };
        
        if params.speciation_enabled {
            // Calculate species statistics
            let species_stats = calculate_species_statistics(generation, previous_species_count);
            
            // Update previous species count for next generation
            {
                let mut prev_count = PREVIOUS_SPECIES_COUNT.lock().unwrap();
                *prev_count = species_stats.num_species;
            }
            
            // Write 13-column format with species statistics
            writeln!(
                file,
                "{} {} {} {} {} {} {} {} {} {} {} {} {}",
                generation,
                number_survivors,
                diversity,
                avg_length,
                murder_count,
                species_stats.num_species,
                species_stats.avg_species_size,
                species_stats.largest_species,
                species_stats.smallest_species,
                species_stats.new_species,
                species_stats.extinct_species,
                species_stats.avg_species_age,
                species_stats.avg_stagnation
            )
            .ok();
        } else {
            // Write 5-column format (backward compatible)
            writeln!(
                file,
                "{} {} {} {} {}",
                generation, number_survivors, diversity, avg_length, murder_count
            )
            .ok();
        }
    }
}

// Print stats about pheromone usage.
pub fn display_signal_use(signals: &Signals, params: &Params) {
    // Check if signal sensors are available
    let signal0_available = (Sensor::Signal0 as usize) < NUM_SENSES
        && (Sensor::Signal0Fwd as usize) < NUM_SENSES
        && (Sensor::Signal0Lr as usize) < NUM_SENSES;

    if !signal0_available {
        return;
    }

    let mut sum = 0u64;
    let mut count = 0u32;

    for x in 0..params.size_x {
        for y in 0..params.size_y {
            let magnitude = signals.get_magnitude(0, crate::basic_types::Coord::new(x as i16, y as i16));
            if magnitude != 0 {
                count += 1;
                sum += magnitude as u64;
            }
        }
    }

    let total_cells = params.size_x as f64 * params.size_y as f64;
    let spread_percent = (count as f64 / total_cells) * 100.0;
    let average = sum as f64 / total_cells;

    println!("Signal spread {:.2}%, average {:.2}", spread_percent, average);
}

// Print how many connections occur from each kind of sensor neuron and to
// each kind of action neuron over the entire population. This helps us to
// see which sensors and actions are most useful for survival.
pub fn display_sensor_action_reference_counts(peeps: &Peeps, params: &Params) {
    let mut sensor_counts = vec![0u32; NUM_SENSES];
    let mut action_counts = vec![0u32; NUM_ACTIONS];

    for index in 1..=params.population {
        if let Some(indiv) = peeps.get(index as u16) {
            if indiv.alive {
                for gene in &indiv.nnet.connections {
                    if gene.source_type == SENSOR {
                        let sensor_num = gene.source_num as usize;
                        if sensor_num < NUM_SENSES {
                            sensor_counts[sensor_num] += 1;
                        }
                    }
                    if gene.sink_type == ACTION {
                        let action_num = gene.sink_num as usize;
                        if action_num < NUM_ACTIONS {
                            action_counts[action_num] += 1;
                        }
                    }
                }
            }
        }
    }

    println!("Sensors in use:");
    for (i, &count) in sensor_counts.iter().enumerate() {
        if count > 0 {
            let sensor: Sensor = unsafe { std::mem::transmute(i as u8) };
            println!("  {} - {}", count, sensor_name(sensor));
        }
    }
    println!("Actions in use:");
    for (i, &count) in action_counts.iter().enumerate() {
        if count > 0 {
            let action: Action = unsafe { std::mem::transmute(i as u8) };
            println!("  {} - {}", count, action_name(action));
        }
    }
}

pub fn display_sample_genomes(
    count: u32,
    peeps: &Peeps,
    params: &Params,
) {
    let mut remaining = count;
    let mut index = 1u32; // indexes start at 1

    while remaining > 0 && index <= params.population {
        if let Some(indiv) = peeps.get(index as u16) {
            if indiv.alive {
                println!("---------------------------");
                println!("Individual ID {}", index);
                print_genome(indiv);
                println!();

                print_igraph_edge_list(indiv);

                println!("---------------------------");
                remaining -= 1;
            }
        }
        index += 1;
    }

    display_sensor_action_reference_counts(peeps, params);
}

