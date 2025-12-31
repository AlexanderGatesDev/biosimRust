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

