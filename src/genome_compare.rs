// genome_compare.rs -- compute similarity of two genomes

use crate::genome_neurons::{Gene, Genome};
use crate::params::Params;
use crate::peeps::Peeps;
use crate::random::random_uint_range;

// Approximate gene match: Has to match same source, sink, with similar weight
pub fn genes_match(g1: &Gene, g2: &Gene) -> bool {
    g1.sink_num == g2.sink_num
        && g1.source_num == g2.source_num
        && g1.sink_type == g2.sink_type
        && g1.source_type == g2.source_type
        && g1.weight == g2.weight
}

// The jaro_winkler_distance() function is adapted from the C version at
// https://github.com/miguelvps/c/blob/master/jarowinkler.c
// under a GNU license, ver. 3. This comparison function is useful if
// the simulator allows genomes to change length, or if genes are allowed
// to relocate to different offsets in the genome. I.e., this function is
// tolerant of gaps, relocations, and genomes of unequal lengths.
pub fn jaro_winkler_distance(genome1: &Genome, genome2: &Genome) -> f32 {
    let s = genome1;
    let a = genome2;

    let mut m = 0i32;
    let mut t = 0i32;
    let mut sl = s.len() as i32;
    let mut al = a.len() as i32;

    const MAX_NUM_GENES_TO_COMPARE: i32 = 20;
    sl = sl.min(MAX_NUM_GENES_TO_COMPARE); // optimization: approximate for long genomes
    al = al.min(MAX_NUM_GENES_TO_COMPARE);

    let mut sflags = vec![0i32; sl as usize];
    let mut aflags = vec![0i32; al as usize];
    let range = (sl.max(al) / 2 - 1).max(0);

    if sl == 0 || al == 0 {
        return 0.0;
    }

    // Calculate matching characters
    for i in 0..al {
        let start_j = (i - range).max(0);
        let end_j = (i + range + 1).min(sl);
        for j in start_j..end_j {
            if genes_match(&a[i as usize], &s[j as usize]) && sflags[j as usize] == 0 {
                sflags[j as usize] = 1;
                aflags[i as usize] = 1;
                m += 1;
                break;
            }
        }
    }

    if m == 0 {
        return 0.0;
    }

    // Calculate character transpositions
    let mut l = 0i32;
    for i in 0..al {
        if aflags[i as usize] == 1 {
            let mut j = l;
            while j < sl {
                if sflags[j as usize] == 1 {
                    l = j + 1;
                    break;
                }
                j += 1;
            }
            if j < sl && !genes_match(&a[i as usize], &s[j as usize]) {
                t += 1;
            }
        }
    }
    t /= 2;

    // Jaro distance
    let dw = ((m as f32 / sl as f32) + (m as f32 / al as f32) + ((m - t) as f32 / m as f32)) / 3.0;

    // Winkler prefix bonus: boost similarity if genomes start similarly
    const MAX_PREFIX_LENGTH: i32 = 4;
    let prefix_length = MAX_PREFIX_LENGTH.min(sl.min(al));
    let mut matching_prefix = 0i32;
    for i in 0..prefix_length {
        if genes_match(&s[i as usize], &a[i as usize]) {
            matching_prefix += 1;
        } else {
            break;
        }
    }

    // Winkler scaling factor (typically 0.1)
    const WINKLER_SCALING: f32 = 0.1;
    let winkler_bonus = WINKLER_SCALING * matching_prefix as f32 * (1.0 - dw);

    (dw + winkler_bonus).min(1.0)
}

// Works only for genomes of equal length
pub fn hamming_distance_bits(genome1: &Genome, genome2: &Genome) -> f32 {
    assert_eq!(genome1.len(), genome2.len());

    let num_elements = genome1.len();
    let bytes_per_element = std::mem::size_of::<Gene>();
    let length_bytes = num_elements * bytes_per_element;
    let length_bits = length_bytes * 8;
    let mut bit_count = 0u32;

    for (gene1, gene2) in genome1.iter().zip(genome2.iter()) {
        // Convert genes to bytes for bit comparison
        // Gene is 6 bytes: source_type (u8), source_num (u8), sink_type (u8), sink_num (u8), weight (i16)
        let g1_bytes: [u8; 6] = unsafe { std::mem::transmute(*gene1) };
        let g2_bytes: [u8; 6] = unsafe { std::mem::transmute(*gene2) };
        
        // Compare all bits
        for (b1, b2) in g1_bytes.iter().zip(g2_bytes.iter()) {
            bit_count += (b1 ^ b2).count_ones();
        }
    }

    // For two completely random bit patterns, about half the bits will differ,
    // resulting in c. 50% match. We will scale that by 2X to make the range
    // from 0 to 1.0. We clip the value to 1.0 in case the two patterns are
    // negatively correlated for some reason.
    let similarity = 1.0 - (2.0 * bit_count as f32 / length_bits as f32).min(1.0);
    similarity
}

// Works only for genomes of equal length
// Compares genes as 32-bit units (treating each gene as a u32 for comparison)
pub fn hamming_distance_bytes(genome1: &Genome, genome2: &Genome) -> f32 {
    assert_eq!(genome1.len(), genome2.len());

    let num_elements = genome1.len();
    let bytes_per_element = std::mem::size_of::<Gene>();
    let length_bytes = num_elements * bytes_per_element;
    let mut byte_count = 0u32;

    // Compare byte by byte across all genes
    for (gene1, gene2) in genome1.iter().zip(genome2.iter()) {
        let g1_bytes: [u8; 6] = unsafe { std::mem::transmute(*gene1) };
        let g2_bytes: [u8; 6] = unsafe { std::mem::transmute(*gene2) };
        for (b1, b2) in g1_bytes.iter().zip(g2_bytes.iter()) {
            if b1 == b2 {
                byte_count += 1;
            }
        }
    }

    byte_count as f32 / length_bytes as f32
}

// Returns 0.0..1.0
pub fn genome_similarity(g1: &Genome, g2: &Genome, params: &Params) -> f32 {
    let similarity;
    
    // If genomes have different lengths, use Jaro-Winkler (method 0) which handles unequal lengths
    // No artificial length penalties - let natural selection handle genome length evolution
    // Length constraints should come from metabolic costs (Phase 2), not algorithmic penalties
    if g1.len() != g2.len() {
        return jaro_winkler_distance(g1, g2);
    }
    
    similarity = match params.genome_comparison_method {
        0 => jaro_winkler_distance(g1, g2),
        1 => hamming_distance_bits(g1, g2),
        2 => hamming_distance_bytes(g1, g2),
        _ => {
            eprintln!("Invalid genome comparison method: {}", params.genome_comparison_method);
            hamming_distance_bits(g1, g2) // Default fallback
        }
    };
    
    similarity
}

// Returns 0.0..1.0
// Samples random pairs of individuals regardless if they are alive or not
pub fn genetic_diversity(peeps: &Peeps, params: &Params) -> f32 {
    if params.population < 2 {
        return 0.0;
    }

    // count limits the number of genomes sampled for performance reasons.
    let count = 1000u32.min(params.population);
    let mut num_samples = 0i32;
    let mut similarity_sum = 0.0f32;

    let mut remaining = count;
    while remaining > 0 {
        let index0 = random_uint_range(1, params.population - 1); // skip first and last elements
        let index1 = index0 + 1;
        
        if let Some(indiv0) = peeps.get(index0 as u16) {
            if let Some(indiv1) = peeps.get(index1 as u16) {
                similarity_sum += genome_similarity(&indiv0.genome, &indiv1.genome, params);
                num_samples += 1;
            }
        }
        remaining -= 1;
    }

    if num_samples == 0 {
        return 0.0;
    }

    let diversity = 1.0 - (similarity_sum / num_samples as f32);
    diversity
}

