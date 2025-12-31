// params.rs
// Global simulator parameters

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum RunMode {
    Stop,
    Run,
    Pause,
    Abort,
}

#[derive(Debug, Clone)]
pub struct Params {
    pub population: u32,
    pub steps_per_generation: u32,
    pub max_generations: u32,
    pub num_threads: u32,
    pub signal_layers: u32,
    pub genome_max_length: u32,
    pub max_number_neurons: u32,
    pub point_mutation_rate: f64,
    pub gene_insertion_deletion_rate: f64,
    pub deletion_ratio: f64,
    pub kill_enable: bool,
    pub sexual_reproduction: bool,
    pub choose_parents_by_fitness: bool,
    pub population_sensor_radius: f32,
    pub signal_sensor_radius: f32,
    pub responsiveness: f32,
    pub responsiveness_curve_k_factor: u32,
    pub long_probe_distance: u32,
    pub short_probe_barrier_distance: u32,
    pub valence_saturation_mag: f32,
    pub save_video: bool,
    pub video_stride: u32,
    pub video_save_first_frames: u32,
    pub video_framerate: u32,
    pub display_scale: u32,
    pub agent_size: u32,
    pub display_enabled: bool,
    pub display_challenge_area: bool,
    pub display_challenge_area_opacity: f32,
    pub display_challenge_area_r: u8,
    pub display_challenge_area_g: u8,
    pub display_challenge_area_b: u8,
    pub genome_analysis_stride: u32,
    pub display_sample_genomes: u32,
    pub genome_comparison_method: u32,
    pub update_graph_log: bool,
    pub update_graph_log_stride: u32,
    pub challenge: u32,
    pub barrier_type: u32,
    pub deterministic: bool,
    pub rng_seed: u32,
    // These must not change after initialization
    pub size_x: u16,
    pub size_y: u16,
    pub genome_initial_length_min: u32,
    pub genome_initial_length_max: u32,
    pub log_dir: String,
    pub image_dir: String,
    pub graph_log_update_command: String,
    // These are updated automatically and not set via the parameter file
    pub parameter_change_generation_number: u32,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            size_x: 128,
            size_y: 128,
            challenge: 6,
            genome_initial_length_min: 24,
            genome_initial_length_max: 24,
            genome_max_length: 300,
            log_dir: "./logs/".to_string(),
            image_dir: "./images/".to_string(),
            population: 3000,
            steps_per_generation: 300,
            max_generations: 200000,
            barrier_type: 0,
            num_threads: 4,
            signal_layers: 1,
            max_number_neurons: 5,
            point_mutation_rate: 0.001,
            gene_insertion_deletion_rate: 0.0,
            deletion_ratio: 0.5,
            kill_enable: false,
            sexual_reproduction: true,
            choose_parents_by_fitness: true,
            population_sensor_radius: 2.5,
            signal_sensor_radius: 2.0,
            responsiveness: 0.5,
            responsiveness_curve_k_factor: 2,
            long_probe_distance: 16,
            short_probe_barrier_distance: 4,
            valence_saturation_mag: 0.5,
            save_video: true,
            video_stride: 25,
            video_save_first_frames: 2,
            video_framerate: 25,
            display_scale: 8,
            agent_size: 4,
            display_enabled: true,
            display_challenge_area: false,
            display_challenge_area_opacity: 0.5,
            display_challenge_area_r: 255,
            display_challenge_area_g: 255,
            display_challenge_area_b: 0,
            genome_analysis_stride: 25,
            display_sample_genomes: 5,
            genome_comparison_method: 1,
            update_graph_log: true,
            update_graph_log_stride: 25,
            deterministic: false,
            rng_seed: 12345678,
            graph_log_update_command: "/usr/bin/gnuplot --persist ./tools/graphlog.gp".to_string(),
            parameter_change_generation_number: 0,
        }
    }
}

pub struct ParamManager {
    priv_params: Params,
    config_filename: String,
    last_mod_time: Option<SystemTime>,
}

impl ParamManager {
    pub fn new() -> Self {
        let mut pm = ParamManager {
            priv_params: Params::default(),
            config_filename: String::new(),
            last_mod_time: None,
        };
        pm.set_defaults();
        pm
    }

    pub fn get_param_ref(&self) -> &Params {
        &self.priv_params
    }

    pub fn set_defaults(&mut self) {
        // Already set by Default trait
    }

    pub fn register_config_file(&mut self, filename: &str) {
        self.config_filename = filename.to_string();
    }

    pub fn update_from_config_file(&mut self, generation_number: u32) {
        if self.config_filename.is_empty() {
            return;
        }

        let path = Path::new(&self.config_filename);
        if !path.exists() {
            eprintln!("Couldn't open config file {}", self.config_filename);
            return;
        }

        let metadata = fs::metadata(path).ok();
        let mod_time = metadata.and_then(|m| m.modified().ok());
        
        // Only read if file has changed
        if let Some(current_mod_time) = mod_time {
            if let Some(last_mod) = self.last_mod_time {
                if current_mod_time == last_mod {
                    return;
                }
            }
            self.last_mod_time = Some(current_mod_time);
        }

        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(mut line) = line {
                    // Remove whitespace
                    line.retain(|c| !c.is_whitespace());
                    
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    if let Some(delimiter_pos) = line.find('=') {
                        let mut name = line[..delimiter_pos].to_string();
                        
                        // Process generation specifier if present
                        if let Some(gen_delim_pos) = name.find('@') {
                            let gen_specifier = &name[gen_delim_pos + 1..];
                            if let Ok(active_from_gen) = gen_specifier.parse::<u32>() {
                                if active_from_gen > generation_number {
                                    continue; // Not active yet
                                } else if active_from_gen == generation_number {
                                    self.priv_params.parameter_change_generation_number = generation_number;
                                }
                                name = name[..gen_delim_pos].to_string();
                            } else {
                                eprintln!("Invalid generation specifier: {}", name);
                                continue;
                            }
                        }

                        let value = line[delimiter_pos + 1..]
                            .split('#')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();

                        self.ingest_parameter(&name.to_lowercase(), &value);
                    }
                }
            }
        } else {
            eprintln!("Couldn't open config file {}", self.config_filename);
        }
    }

    fn ingest_parameter(&mut self, name: &str, val: &str) {
        let is_uint = val.parse::<u32>().is_ok();
        let u_val = val.parse::<u32>().unwrap_or(0);
        let is_float = val.parse::<f64>().is_ok();
        let d_val = val.parse::<f64>().unwrap_or(0.0);
        let is_bool = matches!(val, "0" | "1" | "true" | "false");
        let b_val = matches!(val, "true" | "1");

        match name {
            "sizex" if is_uint && u_val >= 2 && u_val <= u16::MAX as u32 => {
                self.priv_params.size_x = u_val as u16;
            }
            "sizey" if is_uint && u_val >= 2 && u_val <= u16::MAX as u32 => {
                self.priv_params.size_y = u_val as u16;
            }
            "challenge" if is_uint => {
                self.priv_params.challenge = u_val;
            }
            "genomeinitiallengthmin" if is_uint && u_val > 0 => {
                self.priv_params.genome_initial_length_min = u_val;
            }
            "genomeinitiallengthmax" if is_uint && u_val > 0 => {
                self.priv_params.genome_initial_length_max = u_val;
            }
            "logdir" => {
                self.priv_params.log_dir = val.to_string();
            }
            "imagedir" => {
                self.priv_params.image_dir = val.to_string();
            }
            "population" if is_uint && u_val > 0 => {
                self.priv_params.population = u_val;
            }
            "stepspergeneration" if is_uint && u_val > 0 => {
                self.priv_params.steps_per_generation = u_val;
            }
            "maxgenerations" if is_uint && u_val > 0 => {
                self.priv_params.max_generations = u_val;
            }
            "barriertype" if is_uint => {
                self.priv_params.barrier_type = u_val;
            }
            "numthreads" if is_uint && u_val > 0 => {
                self.priv_params.num_threads = u_val;
            }
            "signallayers" if is_uint => {
                self.priv_params.signal_layers = u_val;
            }
            "genomemaxlength" if is_uint && u_val > 0 => {
                self.priv_params.genome_max_length = u_val;
            }
            "maxnumberneurons" if is_uint && u_val > 0 => {
                self.priv_params.max_number_neurons = u_val;
            }
            "pointmutationrate" if is_float && d_val >= 0.0 && d_val <= 1.0 => {
                self.priv_params.point_mutation_rate = d_val;
            }
            "geneinsertiondeletionrate" if is_float && d_val >= 0.0 && d_val <= 1.0 => {
                self.priv_params.gene_insertion_deletion_rate = d_val;
            }
            "deletionratio" if is_float && d_val >= 0.0 && d_val <= 1.0 => {
                self.priv_params.deletion_ratio = d_val;
            }
            "killenable" if is_bool => {
                self.priv_params.kill_enable = b_val;
            }
            "sexualreproduction" if is_bool => {
                self.priv_params.sexual_reproduction = b_val;
            }
            "chooseparentsbyfitness" if is_bool => {
                self.priv_params.choose_parents_by_fitness = b_val;
            }
            "populationsensorradius" if is_float && d_val > 0.0 => {
                self.priv_params.population_sensor_radius = d_val as f32;
            }
            "signalsensorradius" if is_float && d_val > 0.0 => {
                self.priv_params.signal_sensor_radius = d_val as f32;
            }
            "responsiveness" if is_float && d_val >= 0.0 => {
                self.priv_params.responsiveness = d_val as f32;
            }
            "responsivenesscurvekfactor" if is_uint && u_val >= 1 && u_val <= 20 => {
                self.priv_params.responsiveness_curve_k_factor = u_val;
            }
            "longprobedistance" if is_uint && u_val > 0 => {
                self.priv_params.long_probe_distance = u_val;
            }
            "shortprobebarrierdistance" if is_uint && u_val > 0 => {
                self.priv_params.short_probe_barrier_distance = u_val;
            }
            "valencesaturationmag" if is_float && d_val >= 0.0 => {
                self.priv_params.valence_saturation_mag = d_val as f32;
            }
            "savevideo" if is_bool => {
                self.priv_params.save_video = b_val;
            }
            "videostride" if is_uint && u_val > 0 => {
                self.priv_params.video_stride = u_val;
            }
            "videosavefirstframes" if is_uint => {
                self.priv_params.video_save_first_frames = u_val;
            }
            "videoframerate" if is_uint && u_val > 0 => {
                self.priv_params.video_framerate = u_val;
            }
            "displayscale" if is_uint && u_val > 0 => {
                self.priv_params.display_scale = u_val;
            }
            "displayenabled" if is_bool => {
                self.priv_params.display_enabled = b_val;
            }
            "displaychallengearea" if is_bool => {
                self.priv_params.display_challenge_area = b_val;
            }
            "displaychallengeareaopacity" if is_float && d_val >= 0.0 && d_val <= 1.0 => {
                self.priv_params.display_challenge_area_opacity = d_val as f32;
            }
            "displaychallengeareacolor" => {
                // Parse comma-separated RGB values: "255,255,0"
                let parts: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
                if parts.len() == 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[0].parse::<u32>(),
                        parts[1].parse::<u32>(),
                        parts[2].parse::<u32>(),
                    ) {
                        if r <= 255 && g <= 255 && b <= 255 {
                            self.priv_params.display_challenge_area_r = r as u8;
                            self.priv_params.display_challenge_area_g = g as u8;
                            self.priv_params.display_challenge_area_b = b as u8;
                        } else {
                            eprintln!("Invalid RGB values for displaychallengeareacolor (must be 0-255): {}", val);
                        }
                    } else {
                        eprintln!("Invalid RGB format for displaychallengeareacolor (expected R,G,B): {}", val);
                    }
                } else {
                    eprintln!("Invalid RGB format for displaychallengeareacolor (expected R,G,B): {}", val);
                }
            }
            "agentsize" if is_uint && u_val > 0 => {
                self.priv_params.agent_size = u_val;
            }
            "genomeanalysisstride" if val.to_lowercase() == "videostride" => {
                self.priv_params.genome_analysis_stride = self.priv_params.video_stride;
            }
            "genomeanalysisstride" if is_uint && u_val > 0 => {
                self.priv_params.genome_analysis_stride = u_val;
            }
            "displaysamplegenomes" if is_uint => {
                self.priv_params.display_sample_genomes = u_val;
            }
            "genomecomparisonmethod" if is_uint => {
                self.priv_params.genome_comparison_method = u_val;
            }
            "updategraphlog" if is_bool => {
                self.priv_params.update_graph_log = b_val;
            }
            "updategraphlogstride" if val.to_lowercase() == "videostride" => {
                self.priv_params.update_graph_log_stride = self.priv_params.video_stride;
            }
            "updategraphlogstride" if is_uint && u_val > 0 => {
                self.priv_params.update_graph_log_stride = u_val;
            }
            "deterministic" if is_bool => {
                self.priv_params.deterministic = b_val;
            }
            "rngseed" if is_uint => {
                self.priv_params.rng_seed = u_val;
            }
            _ => {
                eprintln!("Invalid param: {} = {}", name, val);
            }
        }
    }

    pub fn check_parameters(&self) {
        if self.priv_params.deterministic && self.priv_params.num_threads != 1 {
            eprintln!("Warning: When deterministic is true, you probably want to set numThreads = 1.");
        }
    }
}

impl Default for ParamManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn params_init(argc: usize, argv: Vec<String>) -> Params {
    let mut param_manager = ParamManager::new();
    
    let config_file = if argc > 1 && !argv[1].is_empty() {
        &argv[1]
    } else {
        "biosim4.ini"
    };
    
    param_manager.register_config_file(config_file);
    param_manager.update_from_config_file(0);
    param_manager.check_parameters();
    
    param_manager.get_param_ref().clone()
}

