// genome_neurons.rs
// Genome and neural network structures

use crate::random::{random_uint, random_uint_range};
use crate::sensors_actions::{NUM_ACTIONS, NUM_SENSES};
use crate::params::Params;
use std::collections::HashMap;

pub const SENSOR: u8 = 1;  // always a source
pub const ACTION: u8 = 1;   // always a sink
pub const NEURON: u8 = 0;   // can be either a source or sink

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Gene {
    pub source_type: u8,  // SENSOR or NEURON (1 bit)
    pub source_num: u8,    // 7 bits
    pub sink_type: u8,     // NEURON or ACTION (1 bit)
    pub sink_num: u8,      // 7 bits
    pub weight: i16,
}

impl Gene {
    pub fn weight_as_float(&self) -> f32 {
        self.weight as f32 / 8192.0
    }

    pub fn make_random_weight() -> i16 {
        (random_uint_range(0, 0xffff) as i32 - 0x8000) as i16
    }
}

pub type Genome = Vec<Gene>;

// Phase 4: Enhanced Connection structure for parallel synapses and dendritic computation
#[derive(Debug, Clone)]
pub struct Connection {
    pub source_type: u8,         // SENSOR or NEURON
    pub source_num: u8,          // Source neuron/sensor index
    pub sink_type: u8,           // NEURON or ACTION
    pub sink_num: u8,            // Sink neuron/action index
    pub weight: f32,             // Connection weight (f32 for precision, Phase 4)
    pub dendrite_id: u16,        // Dendrite identifier for grouping synapses (Phase 4)
    pub receptor_type: u8,       // Receptor type (0=excitatory, 1=inhibitory, Phase 4)
    pub plasticity_trace: f32,   // STDP trace for future plasticity (Phase 4)
}

impl Connection {
    /// Create a new connection from a gene
    pub fn from_gene(gene: &Gene, dendrite_id: u16) -> Self {
        // Determine receptor type from weight sign
        // Positive = excitatory, negative = inhibitory
        let receptor_type = if gene.weight >= 0 { 0 } else { 1 };
        
        Connection {
            source_type: gene.source_type,
            source_num: gene.source_num,
            sink_type: gene.sink_type,
            sink_num: gene.sink_num,
            weight: gene.weight_as_float(),
            dendrite_id,
            receptor_type,
            plasticity_trace: 0.0,
        }
    }
    
    /// Convert connection back to gene (for genome representation)
    pub fn to_gene(&self) -> Gene {
        // Convert f32 weight back to i16 representation
        let weight_i16 = (self.weight * 8192.0).round() as i32;
        let clamped_weight = weight_i16.max(i16::MIN as i32).min(i16::MAX as i32) as i16;
        
        Gene {
            source_type: self.source_type,
            source_num: self.source_num,
            sink_type: self.sink_type,
            sink_num: self.sink_num,
            weight: clamped_weight,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Neuron {
    pub output: f32,
    pub driven: bool,  // undriven neurons have fixed output values
    // Homeostatic plasticity fields (Phase 3)
    pub target_firing_rate: f32,        // Target firing rate (0.0 to 1.0)
    pub current_firing_rate: f32,        // Current average firing rate (exponential moving average)
    pub homeostatic_scale: f32,         // Multiplicative scaling factor for incoming weights
}

#[derive(Debug, Clone)]
pub struct NeuralNet {
    pub connections: Vec<Connection>,  // Phase 4: Use Connection instead of Gene
    pub neurons: Vec<Neuron>,
}

pub fn initial_neuron_output() -> f32 {
    0.5
}

pub fn make_random_gene() -> Gene {
    // source_type: 0 = NEURON, 1 = SENSOR
    // sink_type: 0 = NEURON, 1 = ACTION
    Gene {
        source_type: (random_uint() & 1) as u8,  // 0 or 1
        source_num: random_uint_range(0, 0x7fff) as u8,
        sink_type: (random_uint() & 1) as u8,     // 0 or 1
        sink_num: random_uint_range(0, 0x7fff) as u8,
        weight: Gene::make_random_weight(),
    }
}

pub fn make_random_genome(params: &Params) -> Genome {
    let length = random_uint_range(
        params.genome_initial_length_min,
        params.genome_initial_length_max,
    ) as usize;
    (0..length).map(|_| make_random_gene()).collect()
}

struct Node {
    remapped_number: u16,
    num_outputs: u16,
    num_self_inputs: u16,
    num_inputs_from_sensors_or_other_neurons: u16,
}

fn make_renumbered_connection_list(genome: &Genome, params: &Params) -> Vec<Gene> {
    let mut connection_list = Vec::new();
    for gene in genome {
        let mut conn = *gene;
        
        if conn.source_type == NEURON {
            conn.source_num = (conn.source_num as u32 % params.max_number_neurons) as u8;
        } else {
            conn.source_num = (conn.source_num as u32 % NUM_SENSES as u32) as u8;
        }
        
        if conn.sink_type == NEURON {
            conn.sink_num = (conn.sink_num as u32 % params.max_number_neurons) as u8;
        } else {
            conn.sink_num = (conn.sink_num as u32 % NUM_ACTIONS as u32) as u8;
        }
        
        connection_list.push(conn);
    }
    connection_list
}

fn make_node_list(connection_list: &[Gene], _params: &Params) -> HashMap<u16, Node> {
    let mut node_map = HashMap::new();
    
    for conn in connection_list {
        // Only track neurons, not sensors or actions
        if conn.sink_type == NEURON {
            let sink_num = conn.sink_num as u16;
            let node = node_map.entry(sink_num).or_insert_with(|| Node {
                remapped_number: 0,
                num_outputs: 0,
                num_self_inputs: 0,
                num_inputs_from_sensors_or_other_neurons: 0,
            });
            
            if conn.source_type == NEURON && conn.source_num as u16 == sink_num {
                node.num_self_inputs += 1;
            } else {
                node.num_inputs_from_sensors_or_other_neurons += 1;
            }
        }
        
        if conn.source_type == NEURON {
            let source_num = conn.source_num as u16;
            let node = node_map.entry(source_num).or_insert_with(|| Node {
                remapped_number: 0,
                num_outputs: 0,
                num_self_inputs: 0,
                num_inputs_from_sensors_or_other_neurons: 0,
            });
            node.num_outputs += 1;
        }
    }
    
    node_map
}

fn remove_connections_to_neuron(connections: &mut Vec<Gene>, node_map: &mut HashMap<u16, Node>, neuron_number: u16) {
    let mut to_remove = Vec::new();
    for (idx, conn) in connections.iter().enumerate() {
        if conn.sink_type == NEURON && conn.sink_num as u16 == neuron_number {
            // If source is a neuron, decrement its output count
            if conn.source_type == NEURON {
                if let Some(node) = node_map.get_mut(&(conn.source_num as u16)) {
                    node.num_outputs = node.num_outputs.saturating_sub(1);
                }
            }
            to_remove.push(idx);
        }
    }
    // Remove in reverse order to maintain indices
    for &idx in to_remove.iter().rev() {
        connections.remove(idx);
    }
}

fn cull_useless_neurons(connections: &mut Vec<Gene>, node_map: &mut HashMap<u16, Node>) {
    // Only cull neurons, not sensor-to-action connections
    // Sensor-to-action connections don't involve neurons, so they're never removed
    loop {
        let mut all_done = true;
        let mut to_remove = Vec::new();
        
        for (neuron_num, node) in node_map.iter() {
            // Remove neurons that have no outputs or only feed themselves
            if node.num_outputs == node.num_self_inputs {
                all_done = false;
                to_remove.push(*neuron_num);
            }
        }
        
        if all_done {
            break;
        }
        
        for neuron_num in to_remove {
            // This only removes connections TO the neuron, not sensor-to-action connections
            remove_connections_to_neuron(connections, node_map, neuron_num);
            node_map.remove(&neuron_num);
        }
    }
}

// Genome mutation and generation functions
pub fn random_bit_flip(genome: &mut Genome) {
    use crate::random::{random_uint, random_uint_range, RANDOM_UINT_MAX};
    
    let element_index = random_uint_range(0, genome.len() as u32 - 1) as usize;
    let bit_index8 = 1u8 << random_uint_range(0, 7) as u8;
    
    let chance = random_uint() as f32 / RANDOM_UINT_MAX as f32;
    if chance < 0.2 {
        // sourceType
        genome[element_index].source_type ^= 1;
    } else if chance < 0.4 {
        // sinkType
        genome[element_index].sink_type ^= 1;
    } else if chance < 0.6 {
        // sourceNum
        genome[element_index].source_num ^= bit_index8;
    } else if chance < 0.8 {
        // sinkNum
        genome[element_index].sink_num ^= bit_index8;
    } else {
        // weight
        genome[element_index].weight ^= 1i16 << random_uint_range(1, 15) as i16;
    }
}

pub fn crop_length(genome: &mut Genome, length: usize) {
    use crate::random::{random_uint, RANDOM_UINT_MAX};
    
    if genome.len() > length && length > 0 {
        if (random_uint() as f32 / RANDOM_UINT_MAX as f32) < 0.5 {
            // trim front
            let number_to_trim = genome.len() - length;
            genome.drain(0..number_to_trim);
        } else {
            // trim back
            genome.truncate(length);
        }
    }
}

pub fn random_insert_deletion(genome: &mut Genome, params: &Params) {
    use crate::random::{random_uint, random_uint_range, RANDOM_UINT_MAX};
    
    let probability = params.gene_insertion_deletion_rate as f32;
    if (random_uint() as f32 / RANDOM_UINT_MAX as f32) < probability {
        // Use constant deletion_ratio regardless of genome length
        // No artificial scaling - let natural selection and metabolic costs
        // (Phase 2) regulate genome length evolution
        let deletion_ratio = params.deletion_ratio as f32;
        
        if (random_uint() as f32 / RANDOM_UINT_MAX as f32) < deletion_ratio {
            // deletion
            if genome.len() > 1 {
                let index = random_uint_range(0, genome.len() as u32 - 1) as usize;
                genome.remove(index);
            }
        } else if genome.len() < params.genome_max_length as usize {
            // insertion
            genome.push(make_random_gene());
        }
    }
}

pub fn apply_point_mutations(genome: &mut Genome, params: &Params) {
    use crate::random::{random_uint, RANDOM_UINT_MAX};
    
    let number_of_genes = genome.len();
    for _ in 0..number_of_genes {
        if (random_uint() as f32 / RANDOM_UINT_MAX as f32) < params.point_mutation_rate as f32 {
            random_bit_flip(genome);
        }
    }
}

pub fn generate_child_genome(parent_genomes: &[Genome], params: &Params) -> Genome {
    use crate::random::{random_uint, random_uint_range};
    
    if parent_genomes.is_empty() {
        return make_random_genome(params);
    }
    
    let parent1_idx = if params.choose_parents_by_fitness && parent_genomes.len() > 1 {
        random_uint_range(1, parent_genomes.len() as u32 - 1) as usize
    } else {
        random_uint_range(0, parent_genomes.len() as u32 - 1) as usize
    };
    
    let parent2_idx = if params.choose_parents_by_fitness && parent_genomes.len() > 1 {
        random_uint_range(0, parent1_idx as u32 - 1) as usize
    } else {
        random_uint_range(0, parent_genomes.len() as u32 - 1) as usize
    };
    
    let g1 = &parent_genomes[parent1_idx];
    let g2 = &parent_genomes[parent2_idx];
    
    if g1.is_empty() || g2.is_empty() {
        panic!("invalid genome");
    }
    
    let mut genome = if params.sexual_reproduction {
        let overlay_with_slice_of = |genome: &mut Genome, g_shorter: &Genome| {
            let index0 = random_uint_range(0, g_shorter.len() as u32 - 1) as usize;
            let index1 = random_uint_range(0, g_shorter.len() as u32) as usize;
            let (start, end) = if index0 < index1 {
                (index0, index1)
            } else {
                (index1, index0)
            };
            
            for (i, gene) in g_shorter[start..end].iter().enumerate() {
                if start + i < genome.len() {
                    genome[start + i] = *gene;
                }
            }
        };
        
        if g1.len() > g2.len() {
            let mut new_genome = g1.clone();
            overlay_with_slice_of(&mut new_genome, g2);
            new_genome
        } else {
            let mut new_genome = g2.clone();
            overlay_with_slice_of(&mut new_genome, g1);
            new_genome
        }
    } else {
        g2.clone()
    };
    
    if params.sexual_reproduction {
        // Trim to average length of parents
        let mut sum = g1.len() + g2.len();
        if (sum & 1) != 0 && (random_uint() & 1) != 0 {
            sum += 1;
        }
        crop_length(&mut genome, sum / 2);
    }
    
    random_insert_deletion(&mut genome, params);
    apply_point_mutations(&mut genome, params);
    
    assert!(!genome.is_empty());
    assert!(genome.len() <= params.genome_max_length as usize);
    
    genome
}

// Deprecated: Use genome_compare::genome_similarity instead
// This function is kept for backward compatibility but delegates to the new implementation
pub fn genome_similarity(g1: &Genome, g2: &Genome) -> f32 {
    // Default to method 1 (Hamming bits) if params not available
    // This is a fallback for code that doesn't have params available
    if g1.len() != g2.len() || g1.is_empty() {
        return 0.0;
    }
    
    // Use simple byte-level comparison as fallback
    let mut matches = 0u32;
    for (gene1, gene2) in g1.iter().zip(g2.iter()) {
        if gene1.source_type == gene2.source_type
            && gene1.source_num == gene2.source_num
            && gene1.sink_type == gene2.sink_type
            && gene1.sink_num == gene2.sink_num
            && gene1.weight == gene2.weight
        {
            matches += 1;
        }
    }
    
    matches as f32 / g1.len() as f32
}

pub fn create_wiring_from_genome(nnet: &mut NeuralNet, genome: &Genome, params: &Params) {
    if genome.is_empty() {
        nnet.connections.clear();
        nnet.neurons.clear();
        return;
    }
    
    let mut connection_list = make_renumbered_connection_list(genome, params);
    let mut node_map = make_node_list(&connection_list, params);
    
    cull_useless_neurons(&mut connection_list, &mut node_map);
    
    // Note: sensor-to-action connections are always preserved (they don't involve neurons)
    
    // Renumber neurons sequentially starting at 0
    // Only renumber neurons that have outputs (feed something)
    let mut new_number = 0u16;
    for node in node_map.values_mut() {
        if node.num_outputs != 0 {
            node.remapped_number = new_number;
            new_number += 1;
        } else {
            // Neurons with no outputs shouldn't be in the map after culling, but just in case
            node.remapped_number = 0xffff; // Mark as invalid
        }
    }
    
    nnet.connections.clear();
    
    // Phase 4: Assign dendrite IDs during wiring
    // Group connections by sink neuron to assign dendrite IDs
    // Each unique source->sink pair gets its own dendrite ID
    let mut dendrite_counter: u16 = 0;
    let mut dendrite_map: std::collections::HashMap<(u8, u8, u8, u8), u16> = std::collections::HashMap::new();
    
    // Helper function to get or assign dendrite ID
    let mut get_dendrite_id = |source_type: u8, source_num: u8, sink_type: u8, sink_num: u8| -> u16 {
        let key = (source_type, source_num, sink_type, sink_num);
        *dendrite_map.entry(key).or_insert_with(|| {
            let id = dendrite_counter;
            dendrite_counter += 1;
            id
        })
    };
    
    // First, connections from sensor or neuron to a neuron
    for conn in &connection_list {
        if conn.sink_type == NEURON {
            let mut new_gene = *conn;
            if let Some(node) = node_map.get(&(conn.sink_num as u16)) {
                // Only add if the neuron wasn't culled (has valid remapped_number)
                if node.remapped_number != 0xffff {
                    new_gene.sink_num = node.remapped_number as u8;
                    if new_gene.source_type == NEURON {
                        if let Some(source_node) = node_map.get(&(conn.source_num as u16)) {
                            if source_node.remapped_number != 0xffff {
                                new_gene.source_num = source_node.remapped_number as u8;
                                let dendrite_id = get_dendrite_id(new_gene.source_type, new_gene.source_num, new_gene.sink_type, new_gene.sink_num);
                                nnet.connections.push(Connection::from_gene(&new_gene, dendrite_id));
                            }
                        }
                    } else {
                        // Source is a sensor, so it's always valid
                        let dendrite_id = get_dendrite_id(new_gene.source_type, new_gene.source_num, new_gene.sink_type, new_gene.sink_num);
                        nnet.connections.push(Connection::from_gene(&new_gene, dendrite_id));
                    }
                }
            }
        }
    }
    
    // Then, connections from sensor or neuron to an action
    // These should ALWAYS be preserved (sensor-to-action don't involve neurons)
    for conn in &connection_list {
        if conn.sink_type == ACTION {
            let mut new_gene = *conn;
            if new_gene.source_type == NEURON {
                // Source is a neuron - only add if the neuron wasn't culled
                if let Some(node) = node_map.get(&(conn.source_num as u16)) {
                    if node.remapped_number != 0xffff {
                        new_gene.source_num = node.remapped_number as u8;
                        let dendrite_id = get_dendrite_id(new_gene.source_type, new_gene.source_num, new_gene.sink_type, new_gene.sink_num);
                        nnet.connections.push(Connection::from_gene(&new_gene, dendrite_id));
                    }
                }
            } else {
                // Source is a sensor - always add (sensor-to-action connections)
                // These are never culled because they don't involve neurons
                let dendrite_id = get_dendrite_id(new_gene.source_type, new_gene.source_num, new_gene.sink_type, new_gene.sink_num);
                nnet.connections.push(Connection::from_gene(&new_gene, dendrite_id));
            }
        }
    }
    
    
    // Create the neural node list
    // After renumbering, we have new_number neurons (sequentially numbered 0..new_number-1)
    nnet.neurons.clear();
    for neuron_idx in 0..new_number as usize {
        // Find the node with remapped_number == neuron_idx to get its driven status
        let mut driven = false;
        for node in node_map.values() {
            if node.remapped_number as usize == neuron_idx {
                driven = node.num_inputs_from_sensors_or_other_neurons != 0;
                break;
            }
        }
        nnet.neurons.push(Neuron {
            output: initial_neuron_output(),
            driven,
            // Initialize homeostatic fields (Phase 3)
            target_firing_rate: params.target_firing_rate,
            current_firing_rate: 0.0,
            homeostatic_scale: 1.0,  // Start with no scaling
        });
    }
    
    // Also mark neurons as driven if they receive connections (double-check)
    for conn in &nnet.connections {
        if conn.sink_type == NEURON {
            let neuron_idx = conn.sink_num as usize;
            if neuron_idx < nnet.neurons.len() {
                nnet.neurons[neuron_idx].driven = true;
            }
        }
    }
    
    // Conditionally deduplicate connections based on configuration
    // When allow_duplicate_connections is true, duplicate connections are preserved
    // to enable copy number effects (gene dosage). When false, duplicates are merged
    // by summing weights (legacy behavior for backward compatibility).
    if !params.allow_duplicate_connections {
        deduplicate_connections_phase4(&mut nnet.connections);
    }
}

// Phase 4: Updated deduplication for Connection struct
fn deduplicate_connections_phase4(connections: &mut Vec<Connection>) {
    use std::collections::HashMap;
    
    // Map key: (sourceType, sourceNum, sinkType, sinkNum) -> accumulated weight and dendrite_id
    let mut connection_map: HashMap<(u8, u8, u8, u8), (f32, u16, u8)> = HashMap::new();
    
    // Sum weights for duplicate connections
    for conn in connections.iter() {
        let key = (conn.source_type, conn.source_num, conn.sink_type, conn.sink_num);
        let entry = connection_map.entry(key).or_insert((0.0, conn.dendrite_id, conn.receptor_type));
        entry.0 += conn.weight; // Sum weights
        // Keep first dendrite_id and receptor_type encountered
    }
    
    // Rebuild connections list with deduplicated entries
    connections.clear();
    for ((source_type, source_num, sink_type, sink_num), (weight_sum, dendrite_id, receptor_type)) in connection_map {
        connections.push(Connection {
            source_type,
            source_num,
            sink_type,
            sink_num,
            weight: weight_sum,
            dendrite_id,
            receptor_type,
            plasticity_trace: 0.0,
        });
    }
}

