// simulator.rs - Main simulator logic

use crate::params::{ParamManager, RunMode};
use crate::grid::Grid;
use crate::signals::Signals;
use crate::peeps::Peeps;
use crate::image_writer::ImageWriter;
use crate::end_of_sim_step::end_of_sim_step;
use crate::end_of_generation::end_of_generation;
use crate::spawn_new_generation::{initialize_generation_0, spawn_new_generation};
use crate::analysis::print_sensors_actions;
use crate::analysis::display_sample_genomes;

// Challenge constants
pub const CHALLENGE_CIRCLE: u32 = 0;
pub const CHALLENGE_RIGHT_HALF: u32 = 1;
pub const CHALLENGE_RIGHT_QUARTER: u32 = 2;
pub const CHALLENGE_STRING: u32 = 3;
pub const CHALLENGE_CENTER_WEIGHTED: u32 = 4;
pub const CHALLENGE_CENTER_UNWEIGHTED: u32 = 40;
pub const CHALLENGE_CORNER: u32 = 5;
pub const CHALLENGE_CORNER_WEIGHTED: u32 = 6;
pub const CHALLENGE_MIGRATE_DISTANCE: u32 = 7;
pub const CHALLENGE_CENTER_SPARSE: u32 = 8;
pub const CHALLENGE_LEFT_EIGHTH: u32 = 9;
pub const CHALLENGE_RADIOACTIVE_WALLS: u32 = 10;
pub const CHALLENGE_AGAINST_ANY_WALL: u32 = 11;
pub const CHALLENGE_TOUCH_ANY_WALL: u32 = 12;
pub const CHALLENGE_EAST_WEST_EIGHTHS: u32 = 13;
pub const CHALLENGE_NEAR_BARRIER: u32 = 14;
pub const CHALLENGE_PAIRS: u32 = 15;
pub const CHALLENGE_LOCATION_SEQUENCE: u32 = 16;
pub const CHALLENGE_ALTRUISM: u32 = 17;
pub const CHALLENGE_ALTRUISM_SACRIFICE: u32 = 18;

pub struct Simulator {
    pub param_manager: ParamManager,
    pub grid: Grid,
    pub signals: Signals,
    pub peeps: Peeps,
    pub run_mode: RunMode,
    pub image_writer: ImageWriter,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator {
            param_manager: ParamManager::new(),
            grid: Grid::new(),
            signals: Signals::new(),
            peeps: Peeps::new(),
            run_mode: RunMode::Stop,
            image_writer: ImageWriter::new(),
        }
    }

    pub fn init(&mut self, argc: usize, argv: Vec<String>) {
        let config_file = if argc > 1 && !argv[1].is_empty() {
            &argv[1]
        } else {
            "biosimrust.ini"
        };
        
        self.param_manager.register_config_file(config_file);
        self.param_manager.update_from_config_file(0);
        self.param_manager.check_parameters();
        
        let params = self.param_manager.get_param_ref();
        
        // Initialize rayon thread pool with the configured number of threads
        rayon::ThreadPoolBuilder::new()
            .num_threads(params.num_threads as usize)
            .build_global()
            .expect("Failed to initialize rayon thread pool");
        
        // Initialize random number generator for main thread
        // This must be done before generating any random genomes
        crate::random::initialize_random(params.deterministic, params.rng_seed, 0);
        
        self.grid.init(params.size_x, params.size_y);
        self.signals.init(params.signal_layers as u16, params.size_x, params.size_y);
        self.peeps.init(params.population);
    }

    pub fn initialize_generation_0(&mut self) {
        let params = self.param_manager.get_param_ref();
        initialize_generation_0(&mut self.grid, &mut self.signals, &mut self.peeps, params);
    }

    pub fn simulator(&mut self, argc: usize, argv: Vec<String>) {
        self.init(argc, argv);
        
        // Check display_enabled right after init
        let params = self.param_manager.get_param_ref();
        if params.display_enabled {
            println!("Display enabled: {}x{} (scale: {})", params.size_x, params.size_y, params.display_scale);
        }
        let _ = params; // Release the borrow
        
        print_sensors_actions(); // Show the agents' capabilities
        
        self.initialize_generation_0();
        self.run_mode = RunMode::Run;
        
        let mut generation = 0u32;
        
        loop {
            let params = self.param_manager.get_param_ref();
            if !matches!(self.run_mode, RunMode::Run) || generation >= params.max_generations {
                break;
            }
            let steps_per_generation = params.steps_per_generation;
            let population = params.population;
            let display_enabled = params.display_enabled;
            let _ = params; // Release the borrow
            
            let mut murder_count = 0u32;
            
            for sim_step in 0..steps_per_generation {
                
                // Parallel execution of sim_step_one_indiv
                // Use unsafe to get mutable raw pointers for thread-local access
                let peeps_ptr = &mut self.peeps as *mut Peeps;
                let grid_ptr = &self.grid as *const Grid;
                let signals_ptr = &mut self.signals as *mut Signals;
                let param_manager_ptr = &self.param_manager as *const ParamManager;
                
                // Convert raw pointers to usize (which is definitely Send) for parallel execution
                let peeps_ptr_usize = peeps_ptr as usize;
                let grid_ptr_usize = grid_ptr as usize;
                let signals_ptr_usize = signals_ptr as usize;
                let param_manager_ptr_usize = param_manager_ptr as usize;
                
                // Use rayon::scope to create a parallel scope
                rayon::scope(|s| {
                    for index in 1..=population {
                        // Copy pointer values for each thread
                        let peeps_ptr_usize = peeps_ptr_usize;
                        let grid_ptr_usize = grid_ptr_usize;
                        let signals_ptr_usize = signals_ptr_usize;
                        let param_manager_ptr_usize = param_manager_ptr_usize;
                        let sim_step_local = sim_step;
                        
                        s.spawn(move |_| {
                            // Convert usize back to raw pointers
                            // Safety: Each thread operates on a unique `indiv` from `peeps`
                            // and `signals` is accessed with thread-safe `increment`
                            // `grid` and `param_manager` are read-only
                            // Raw pointers are safe here because each thread accesses a different index
                            unsafe {
                                let peeps_ptr = peeps_ptr_usize as *mut Peeps;
                                let grid_ptr = grid_ptr_usize as *const Grid;
                                let signals_ptr = signals_ptr_usize as *mut Signals;
                                let param_manager_ptr = param_manager_ptr_usize as *const ParamManager;
                                
                                let indiv_ptr = (*peeps_ptr).individuals.as_mut_ptr().add(index as usize);
                                let indiv = &mut *indiv_ptr;
                                if indiv.alive {
                                    let grid_ref = &*grid_ptr;
                                    let signals_ref = &mut *signals_ptr;
                                    let param_manager_ref = &*param_manager_ptr;
                                    let params_ref = param_manager_ref.get_param_ref();
                                    let peeps_ref = &*peeps_ptr; // Immutable reference for feed_forward

                                    // Initialize thread-local RNG
                                    crate::random::initialize_random(params_ref.deterministic, params_ref.rng_seed, rayon::current_thread_index().unwrap_or(0));

                                    indiv.age += 1;

                                    let action_levels = indiv.feed_forward(
                                        sim_step_local,
                                        grid_ref,
                                        signals_ref,
                                        params_ref,
                                        peeps_ref,
                                    );

                                    let _ = params_ref;

                                    // execute_actions needs peeps to queue movements
                                    let params_ref = param_manager_ref.get_param_ref();
                                    crate::execute_actions::execute_actions(
                                        indiv,
                                        &action_levels,
                                        params_ref,
                                        peeps_ref,
                                        signals_ref,
                                    );
                                }
                            }
                        });
                    }
                });
                
                // End of sim step processing
                let params = self.param_manager.get_param_ref();
                murder_count += self.peeps.death_queue_size() as u32;
                
                // Always call end_of_sim_step - it will decide whether to generate frames
                end_of_sim_step(
                    sim_step,
                    generation,
                    params,
                    &mut self.peeps,
                    &mut self.grid,
                    &mut self.signals,
                    &mut self.image_writer,
                );
                
                // Process window events if display is enabled
                if display_enabled {
                    if let Ok(mut window_opt) = self.image_writer.window.lock() {
                        if let Some(ref mut win) = *window_opt {
                            if !win.is_open() {
                                *window_opt = None;
                            }
                        }
                    }
                }
            }
            
            // End of generation processing
            let params = self.param_manager.get_param_ref();
            end_of_generation(generation, params, &mut self.image_writer);
            
            self.param_manager.update_from_config_file(generation + 1);
            let params = self.param_manager.get_param_ref();
            
            let number_survivors = spawn_new_generation(
                generation,
                murder_count,
                &mut self.grid,
                &mut self.signals,
                &mut self.peeps,
                params,
            );
            
            if number_survivors > 0 && (generation % params.genome_analysis_stride == 0) {
                display_sample_genomes(params.display_sample_genomes, &self.peeps, params);
            }
            
            if number_survivors == 0 {
                generation = 0; // Start over
            } else {
                generation += 1;
            }
        }
        
        let params = self.param_manager.get_param_ref();
        display_sample_genomes(3, &self.peeps, params); // Final report
        
        println!("Simulator exit.");
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

