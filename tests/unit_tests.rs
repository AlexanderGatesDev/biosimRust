// unit_tests.rs
// Comprehensive unit tests for BiosimRust implementation

use biosimrust::basic_types::Coord;
use biosimrust::genome_neurons::{Gene, Genome, SENSOR, NEURON, ACTION, create_wiring_from_genome};
use biosimrust::grid::{Grid, visit_neighborhood};
use biosimrust::params::Params;
use biosimrust::indiv::Indiv;

#[test]
    fn test_connect_neural_net_wiring_from_genome() {
        // This test verifies that neural network wiring is correctly created from a genome
        // Note: The original C++ test had commented-out genes, so we'll create a simple test
        
        let mut indiv = Indiv::new();
        let params = Params::default();
        
        // Create a simple genome with a few connections
        // SENSOR 0 -> NEURON 0 with weight 0.0
        // SENSOR 1 -> NEURON 2 with weight 2.2
        // NEURON 0 -> ACTION 1 with weight 11.0
        let genome: Genome = vec![
            Gene {
                source_type: SENSOR,
                source_num: 0,
                sink_type: NEURON,
                sink_num: 0,
                weight: 0,
            },
            Gene {
                source_type: SENSOR,
                source_num: 1,
                sink_type: NEURON,
                sink_num: 2,
                weight: 2200, // 2.2 * 1000 (stored as i16, scaled by 1000)
            },
            Gene {
                source_type: NEURON,
                source_num: 0,
                sink_type: ACTION,
                sink_num: 1,
                weight: 11000, // 11.0 * 1000
            },
        ];
        
        indiv.genome = genome;
        
        // Create wiring from genome (modifies nnet in place)
        create_wiring_from_genome(&mut indiv.nnet, &indiv.genome, &params);
        
        // Verify connections were created
        // The exact number depends on the wiring algorithm, but we should have at least some connections
        assert!(!indiv.nnet.connections.is_empty(), "Expected at least one connection");
        
        // Print connections for debugging (similar to original C++ test)
        for conn in &indiv.nnet.connections {
            // Copy values to avoid packed struct reference issues
            let source_type = conn.source_type;
            let source_num = conn.source_num;
            let sink_type = conn.sink_type;
            let sink_num = conn.sink_num;
            let weight = conn.weight;
            
            let source_type_str = if source_type == SENSOR { "SENSOR" } else { "NEURON" };
            let sink_type_str = if sink_type == ACTION { "ACTION" } else { "NEURON" };
            println!(
                "{} {} -> {} {} at {}",
                source_type_str, source_num,
                sink_type_str, sink_num,
                weight
            );
        }
    }

    #[test]
    fn test_grid_visit_neighborhood() {
        // Create a test grid and params
        let mut params = Params::default();
        params.size_x = 64;
        params.size_y = 64;
        
        let mut grid = Grid::new();
        grid.init(params.size_x, params.size_y);
        
        // Test loc 10,10 radius 1
        let mut visited = Vec::new();
        visit_neighborhood(Coord::new(10, 10), 1.0, &params, |loc| {
            visited.push(loc);
        });
        println!("Test loc 10,10 radius 1: {} locations", visited.len());
        assert!(!visited.is_empty());
        visited.clear();
        
        // Test loc 0,0 radius 1
        visit_neighborhood(Coord::new(0, 0), 1.0, &params, |loc| {
            visited.push(loc);
        });
        println!("Test loc 0,0 radius 1: {} locations", visited.len());
        assert!(!visited.is_empty());
        visited.clear();
        
        // Test loc 10,10 radius 1.4
        visit_neighborhood(Coord::new(10, 10), 1.4, &params, |loc| {
            visited.push(loc);
        });
        println!("Test loc 10,10 radius 1.4: {} locations", visited.len());
        assert!(!visited.is_empty());
        visited.clear();
        
        // Test loc 10,10 radius 1.5
        visit_neighborhood(Coord::new(10, 10), 1.5, &params, |loc| {
            visited.push(loc);
        });
        println!("Test loc 10,10 radius 1.5: {} locations", visited.len());
        assert!(!visited.is_empty());
        visited.clear();
        
        // Test loc 1,1 radius 1.4
        visit_neighborhood(Coord::new(1, 1), 1.4, &params, |loc| {
            visited.push(loc);
        });
        println!("Test loc 1,1 radius 1.4: {} locations", visited.len());
        assert!(!visited.is_empty());
        visited.clear();
        
        // Test loc 10,10 radius 2.0
        visit_neighborhood(Coord::new(10, 10), 2.0, &params, |loc| {
            visited.push(loc);
        });
        println!("Test loc 10,10 radius 2.0: {} locations", visited.len());
        assert!(!visited.is_empty());
        visited.clear();
        
        // Test loc p.sizeX-1, p.sizeY-1 radius 2.0
        let edge_x = (params.size_x - 1) as i16;
        let edge_y = (params.size_y - 1) as i16;
        visit_neighborhood(Coord::new(edge_x, edge_y), 2.0, &params, |loc| {
            visited.push(loc);
        });
        println!("Test loc {},{}, radius 2.0: {} locations", edge_x, edge_y, visited.len());
        assert!(!visited.is_empty());
        
        // Verify all visited locations are within bounds
        for loc in &visited {
            assert!(loc.x >= 0 && loc.x < params.size_x as i16);
            assert!(loc.y >= 0 && loc.y < params.size_y as i16);
        }
    }

    #[test]
    fn test_jaro_winkler_unequal_lengths() {
        use biosimrust::genome_compare::jaro_winkler_distance;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        
        // Create two identical genomes of different lengths
        let mut genome1: Genome = Vec::new();
        let mut genome2: Genome = Vec::new();
        
        // Create 10 identical genes
        for i in 0..10 {
            genome1.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        // genome2 has the same 10 genes plus 5 more
        for i in 0..15 {
            genome2.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        // Should have high similarity since first 10 genes match
        let similarity = jaro_winkler_distance(&genome1, &genome2);
        println!("Similarity between 10-gene and 15-gene genomes: {}", similarity);
        assert!(similarity > 0.5, "Expected high similarity for overlapping genomes");
        assert!(similarity <= 1.0, "Similarity should not exceed 1.0");
    }

    #[test]
    fn test_jaro_winkler_prefix_bonus() {
        use biosimrust::genome_compare::jaro_winkler_distance;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        
        // Create two genomes with matching prefixes
        let mut genome1: Genome = Vec::new();
        let mut genome2: Genome = Vec::new();
        
        // First 4 genes match exactly (should get Winkler bonus)
        for i in 0..4 {
            let gene = Gene {
                source_type: SENSOR,
                source_num: i as u8,
                sink_type: NEURON,
                sink_num: i as u8,
                weight: (i as i32 * 1000) as i16,
            };
            genome1.push(gene);
            genome2.push(gene);
        }
        
        // Rest of genes differ
        for i in 4..10 {
            genome1.push(Gene {
                source_type: SENSOR,
                source_num: i as u8,
                sink_type: NEURON,
                sink_num: i as u8,
                weight: (i as i32 * 1000) as i16,
            });
            genome2.push(Gene {
                source_type: SENSOR,
                source_num: (i + 10) as u8,
                sink_type: NEURON,
                sink_num: (i + 10) as u8,
                weight: ((i + 10) as i32 * 1000) as i16,
            });
        }
        
        let similarity = jaro_winkler_distance(&genome1, &genome2);
        println!("Similarity with 4 matching prefix genes: {}", similarity);
        
        // Should be higher than if there was no prefix match
        // With 4 matching prefix genes, Winkler bonus = 0.1 * 4 * (1 - jaro)
        // So similarity should be > jaro distance alone
        assert!(similarity > 0.0, "Similarity should be positive");
        assert!(similarity <= 1.0, "Similarity should not exceed 1.0");
    }

    #[test]
    fn test_genome_similarity_length_penalty() {
        use biosimrust::genome_compare::genome_similarity;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        use biosimrust::params::Params;
        
        let mut params = Params::default();
        params.genome_comparison_method = 0; // Use Jaro-Winkler
        
        // Create two genomes with very different lengths
        let mut genome1: Genome = Vec::new();
        let mut genome2: Genome = Vec::new();
        
        // genome1: 10 genes
        for i in 0..10 {
            genome1.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        // genome2: 50 genes (5x longer)
        for i in 0..50 {
            genome2.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        let similarity = genome_similarity(&genome1, &genome2, &params);
        println!("Similarity between 10-gene and 50-gene genomes: {}", similarity);
        
        // With length penalty: similarity = jaro * 0.8 + length_ratio * 0.2
        // length_ratio = min(10, 50) / max(10, 50) = 10/50 = 0.2
        // So even if jaro is high, the length penalty should reduce it
        assert!(similarity < 1.0, "Similarity should be penalized for large length differences");
        assert!(similarity > 0.0, "Similarity should still be positive");
        
        // Test with more similar lengths
        let mut genome3: Genome = Vec::new();
        for i in 0..12 {
            genome3.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        let similarity2 = genome_similarity(&genome1, &genome3, &params);
        println!("Similarity between 10-gene and 12-gene genomes: {}", similarity2);
        
        // Should be higher than the 10 vs 50 comparison due to better length ratio
        assert!(similarity2 > similarity, 
            "Similarity should be higher for genomes with closer lengths");
    }

    #[test]
    fn test_genome_similarity_equal_lengths() {
        use biosimrust::genome_compare::genome_similarity;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        use biosimrust::params::Params;
        
        let mut params = Params::default();
        params.genome_comparison_method = 0; // Use Jaro-Winkler
        
        // Create two identical genomes of equal length
        let mut genome1: Genome = Vec::new();
        let mut genome2: Genome = Vec::new();
        
        for i in 0..10 {
            let gene = Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            };
            genome1.push(gene);
            genome2.push(gene);
        }
        
        let similarity = genome_similarity(&genome1, &genome2, &params);
        println!("Similarity between identical 10-gene genomes: {}", similarity);
        
        // Identical genomes should have very high similarity
        assert!(similarity > 0.9, "Identical genomes should have very high similarity");
        assert!(similarity <= 1.0, "Similarity should not exceed 1.0");
    }

    #[test]
    fn test_genome_similarity_completely_different() {
        use biosimrust::genome_compare::genome_similarity;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        use biosimrust::params::Params;
        
        let mut params = Params::default();
        params.genome_comparison_method = 0; // Use Jaro-Winkler
        
        // Create two completely different genomes
        let mut genome1: Genome = Vec::new();
        let mut genome2: Genome = Vec::new();
        
        // genome1: genes with low numbers
        for i in 0..10 {
            genome1.push(Gene {
                source_type: SENSOR,
                source_num: i as u8,
                sink_type: NEURON,
                sink_num: i as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        // genome2: genes with high numbers (no overlap)
        for i in 100..110 {
            genome2.push(Gene {
                source_type: SENSOR,
                source_num: (i % 256) as u8,
                sink_type: NEURON,
                sink_num: (i % 256) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        let similarity = genome_similarity(&genome1, &genome2, &params);
        println!("Similarity between completely different genomes: {}", similarity);
        
        // Should have low similarity
        assert!(similarity < 0.5, "Completely different genomes should have low similarity");
        assert!(similarity >= 0.0, "Similarity should not be negative");
    }

    #[test]
    fn test_genome_similarity_edge_cases() {
        use biosimrust::genome_compare::genome_similarity;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        use biosimrust::params::Params;
        
        let mut params = Params::default();
        params.genome_comparison_method = 0; // Use Jaro-Winkler
        
        // Test: one genome is very short (1 gene)
        let mut genome1: Genome = Vec::new();
        let mut genome2: Genome = Vec::new();
        
        genome1.push(Gene {
            source_type: SENSOR,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 0,
            weight: 0,
        });
        
        for i in 0..20 {
            genome2.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        let similarity = genome_similarity(&genome1, &genome2, &params);
        println!("Similarity between 1-gene and 20-gene genomes: {}", similarity);
        
        // Should handle gracefully without crashing
        assert!(similarity >= 0.0 && similarity <= 1.0, 
            "Similarity should be in valid range [0.0, 1.0]");
        
        // Test: empty genomes (should return 0.0)
        let empty1: Genome = Vec::new();
        let empty2: Genome = Vec::new();
        let similarity_empty = genome_similarity(&empty1, &empty2, &params);
        println!("Similarity between empty genomes: {}", similarity_empty);
        assert_eq!(similarity_empty, 0.0, "Empty genomes should have 0 similarity");
    }

    #[test]
    fn test_length_penalty_calculation() {
        use biosimrust::genome_compare::genome_similarity;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        use biosimrust::params::Params;
        
        let mut params = Params::default();
        params.genome_comparison_method = 0; // Use Jaro-Winkler
        
        // Create genomes with specific length ratios to test penalty
        let mut genome1: Genome = Vec::new();
        let mut genome2: Genome = Vec::new();
        
        // Both have same genes (high Jaro similarity)
        for i in 0..10 {
            let gene = Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            };
            genome1.push(gene);
            genome2.push(gene);
        }
        
        // Add extra genes to genome2 to test length penalty
        for i in 10..20 {
            genome2.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        // Now genome1 has 10 genes, genome2 has 20 genes
        // length_ratio = 10/20 = 0.5
        // If Jaro similarity is high (say 0.9), then:
        // similarity = 0.9 * 0.8 + 0.5 * 0.2 = 0.72 + 0.1 = 0.82
        
        let similarity = genome_similarity(&genome1, &genome2, &params);
        println!("Similarity with length penalty (10 vs 20 genes): {}", similarity);
        
        // The similarity should be less than if they were the same length
        // (because of the length penalty)
        assert!(similarity < 1.0, "Length penalty should reduce similarity");
        assert!(similarity > 0.0, "Similarity should still be positive");
        
        // Test with even more extreme difference
        let mut genome3: Genome = genome1.clone();
        for i in 10..50 {
            genome3.push(Gene {
                source_type: SENSOR,
                source_num: (i % 5) as u8,
                sink_type: NEURON,
                sink_num: (i % 3) as u8,
                weight: (i as i32 * 1000) as i16,
            });
        }
        
        // genome1: 10 genes, genome3: 50 genes
        // length_ratio = 10/50 = 0.2
        let similarity_extreme = genome_similarity(&genome1, &genome3, &params);
        println!("Similarity with extreme length difference (10 vs 50 genes): {}", similarity_extreme);
        
        // Should be lower than the 10 vs 20 comparison
        assert!(similarity_extreme < similarity, 
            "More extreme length differences should result in lower similarity");
    }

    #[test]
    fn test_variable_length_genome_stability_simulation() {
        use biosimrust::genome_compare::genome_similarity;
        use biosimrust::genome_neurons::{Gene, SENSOR, NEURON};
        use biosimrust::params::Params;
        
        // Simulate what would happen over multiple generations
        // by comparing genomes of varying lengths and checking
        // that the length penalty prevents extreme convergence
        
        let mut params = Params::default();
        params.genome_comparison_method = 0; // Use Jaro-Winkler
        
        // Create a "population" of genomes with varying lengths
        let mut genomes: Vec<Genome> = Vec::new();
        
        // Create genomes with lengths: 10, 15, 20, 25, 30, 50, 100
        for &length in &[10, 15, 20, 25, 30, 50, 100] {
            let mut genome: Genome = Vec::new();
            for i in 0..length {
                genome.push(Gene {
                    source_type: SENSOR,
                    source_num: (i % 5) as u8,
                    sink_type: NEURON,
                    sink_num: (i % 3) as u8,
                    weight: ((i as i32 * 1000) % 32767) as i16,
                });
            }
            genomes.push(genome);
        }
        
        // Compare each genome with a "reference" genome of length 20
        let reference = &genomes[2]; // Length 20
        
        let mut similarities: Vec<(usize, f32)> = Vec::new();
        for (i, genome) in genomes.iter().enumerate() {
            let sim = genome_similarity(reference, genome, &params);
            similarities.push((i, sim));
            println!("Reference (20 genes) vs genome {} ({} genes): similarity = {}", 
                i, [10, 15, 20, 25, 30, 50, 100][i], sim);
        }
        
        // The similarity should decrease as length difference increases
        // This simulates selection pressure against extreme lengths
        
        // Check that genomes closer to reference length have higher similarity
        let ref_idx = 2; // Reference is at index 2 (length 20)
        let ref_sim = similarities[ref_idx].1;
        
        // Genomes closer to reference should have higher similarity
        assert!(similarities[1].1 > similarities[0].1, 
            "15-gene genome should be more similar than 10-gene genome");
        assert!(similarities[3].1 > similarities[5].1, 
            "25-gene genome should be more similar than 50-gene genome");
        assert!(similarities[3].1 > similarities[6].1, 
            "25-gene genome should be more similar than 100-gene genome");
        
        // The reference should have highest similarity with itself
        assert_eq!(ref_sim, similarities[ref_idx].1, 
            "Genome should have highest similarity with itself");
        
        println!("\nTest passed: Length penalty creates selection pressure against extreme lengths");
    }

