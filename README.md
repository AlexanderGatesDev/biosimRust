# BiosimRust

## Status

This is a complete Rust refactoring of the original C++ biosim4 project, renamed to BiosimRust. The Rust implementation maintains full feature parity with the original while providing improved safety, performance, and maintainability through Rust's type system and memory safety guarantees.

The original C++ implementation can be found here https://github.com/davidrmiller/biosim4 for reference.

## What is this?

This program simulates biological creatures that evolve through natural selection in a 2D environment. The creatures have neural network brains that process sensory inputs and produce actions. Over generations, they evolve to survive various challenges.

The results of the experiments are summarized in this YouTube video:

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;"I programmed some creatures. They evolved."

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;https://www.youtube.com/watch?v=N3tRFayqVtk

This command line program provides a simulation environment where you can observe evolution in action. The program can generate videos of the simulation and real-time visualization windows.

## Document Contents

-  [Code walkthrough](#CodeWalkthrough)
-  [Main data structures](#MainDataStructures)
-  [Config file](#ConfigFile)
-  [Program output](#ProgramOutput)
-  [Main program loop](#MainProgramLoop)
-  [Sensory inputs and action outputs](#SensoryInputsAndActionOutputs)
-  [Basic value types](#BasicValueTypes)
-  [Pheromones](#Pheromones)
-  [Useful utility functions](#UsefulUtilityFunctions)
-  [Installing the code](#InstallingTheCode)
-  [Building the executable](#BuildingTheExecutable)
-  [System requirements](#SystemRequirements)
-  [Compiling](#Compiling)
-  [Execution](#Execution)
-  [Testing](#Testing)
-  [Tools directory](#ToolsDirectory)
-  [Rust vs C++ Implementation](#RustVsCpp)

## Code walkthrough<a name="CodeWalkthrough"></a>

<a name="MainDataStructures"></a>

### Main data structures

The code in the `src/` directory compiles to a single console program named `biosimrust`. When it is invoked, it will read parameters from a config file named `biosimrust.ini` by default. A different config file can be specified on the command line.

The simulator configures a 2D arena where the creatures live. The `Grid` struct (see `src/grid.rs`) contains a 2D array of 16-bit indexes, where each nonzero index refers to a specific individual in the `Peeps` collection (see below). Zero values in `Grid` indicate empty locations. The `Grid` struct does not know anything else about the world; it only stores indexes to represent who lives where.

The population of creatures is stored in the `Peeps` struct (see `src/peeps.rs`). `Peeps` contains all the individuals in the simulation, stored as instances of `Indiv` in a `Vec` container. The indexes in `Grid` are indexes into the vector of individuals in `Peeps`. `Peeps` keeps a container of `Indiv`, but otherwise does not know anything about the internal workings of individuals.

Each individual is represented by an instance of `Indiv` (see `src/indiv.rs`). `Indiv` contains an individual's genome, its corresponding neural net brain, and a redundant copy of the individual's X,Y location in the 2D grid. It also contains a few other parameters for the individual, such as its "responsiveness" level, oscillator period, age, and other personal parameters. `Indiv` knows how to convert an individual's genome into its neural net brain at the beginning of the simulation. It also knows how to print the genome and neural net brain in text format to stdout during a simulation. It has a function `get_sensor()` that is called to compute the individual's input neurons for each simulator step.

All the simulator code lives in the `biosimrust` crate (Rust modules).

<a name="ConfigFile"></a>

### Config file

The config file, named `biosimrust.ini` by default, contains all the tunable parameters for a simulation run. The `biosimrust` executable reads the config file at startup, then monitors it for changes during the simulation. Although it's not foolproof, many parameters can be modified during the simulation run. The `ParamManager` struct (see `src/params.rs`) manages the configuration parameters and makes them available to the simulator through a read-only reference provided by `ParamManager::get_param_ref()`.

See the provided `biosimrust.ini` for documentation for each parameter. Most of the parameters in the config file correspond to members in the `Params` struct (see `src/params.rs`). A few additional parameters may be stored in `Params`. See the documentation in `params.rs` for how to support new parameters.

<a name="ProgramOutput"></a>

### Program output

Depending on the parameters in the config file, the following data can be produced:

-  The simulator will append one line to `logs/epoch-log.txt` after the completion of each generation. Each line records the generation number, number of individuals who survived the selection criterion, an estimate of the population's genetic diversity, average genome length, and number of deaths due to the "kill" gene. This file can be fed to `tools/graphlog.gp` to produce a graphic plot.

-  The simulator will display a small number of sample genomes at regular intervals to stdout. Parameters in the config file specify the number and interval. The genomes are displayed in hex format and also in a mnemonic format that can be fed to `tools/graph-nnet.py` to produce a graphic network diagram.

-  Movies of selected generations will be created in the `images/` directory. Parameters in the config file specify the interval at which to make movies. Each movie records a single generation. The simulator uses FFmpeg to assemble PNG frames into MP4 videos.

-  Real-time visualization is available when `displayenabled = true` in the config file. A window will open showing the simulation as it runs, with optional challenge area highlighting.

-  At intervals, a summary is printed to stdout showing the total number of neural connections throughout the population from each possible sensory input neuron and to each possible action output neuron.

<a name="MainProgramLoop"></a>

### Main program loop

The simulator starts with a call to `simulator()` in `src/simulator.rs`. After initializing the world, the simulator executes three nested loops: the outer loop for each generation, an inner loop for each simulator step within the generation, and an innermost loop for each individual in the population. The innermost loop is parallelized using the `rayon` crate for multi-threaded execution.

At the end of each simulator step, a call is made to `end_of_sim_step()` in single-thread mode (see `src/end_of_sim_step.rs`) to create a video frame representing the locations of all the individuals at the end of the simulator step. The video frame is saved immediately as a PNG file and can be displayed in real-time if enabled. Also some housekeeping may be done for certain selection scenarios. See the comments in `end_of_sim_step.rs` for more information.

At the end of each generation, a call is made to `end_of_generation()` in single-thread mode (see `src/end_of_generation.rs`) to create a video from the saved PNG frames using FFmpeg. Also a new graph might be generated showing the progress of the simulation. See `end_of_generation.rs` for more information.

<a name="SensoryInputsAndActionOutputs"></a>

### Sensory inputs and action outputs

See the YouTube video (link above) for a description of the sensory inputs and action outputs. Each sensory input and each action output is a neuron in the individual's neural net brain.

The module `src/sensors_actions.rs` contains enum `Sensor` which enumerates all the possible sensory inputs and enum `Action` which enumerates all the possible action outputs. In enum `Sensor`, all the sensory inputs before the enumerant `NUM_SENSES` will be compiled into the executable, and all action outputs before `NUM_ACTIONS` will be compiled. By rearranging the enumerants in those enums, you can select a subset of all possible sensory and action neurons to be compiled into the simulator.

<a name="BasicValueTypes"></a>

### Basic value types

There are a few basic value types:

-  `enum Compass` represents eight-way directions with enumerants N, NE, E, SE, S, SW, W, NW, CENTER.

-  `struct Dir` is an abstract representation of the values of `enum Compass`.

-  `struct Coord` is a signed 16-bit integer X,Y coordinate pair. It is used to represent a location in the 2D world, or can represent the difference between two locations.

-  `struct Polar` holds a signed 32-bit integer magnitude and a direction of type `Dir`.

Various conversions and math are possible between these basic types. See the unit tests in `src/basic_types.rs` for examples. Also see `src/basic_types.rs` for more information.

<a name="Pheromones"></a>

### Pheromones

A simple system is used to simulate pheromones emitted by the individuals. Pheromones are called "signals" in simulator-speak (see `src/signals.rs`). `Signals` holds multiple layers that overlay the 2D world in `Grid`. Each location can contain a level of pheromone. The pheromone level at any grid location is stored as an unsigned 8-bit integer, where zero means no pheromone, and 255 is the maximum. Each time an individual emits a pheromone, it increases the pheromone values in a small neighborhood around the individual up to the maximum value of 255. Pheromone levels decay over time if they are not replenished by the individuals in the area.

<a name="UsefulUtilityFunctions"></a>

### Useful utility functions

The utility function `visit_neighborhood()` in `src/grid.rs` can be used to execute a user-defined closure over each location within a circular neighborhood defined by a center point and floating point radius. The function calls the user-defined closure once for each location, passing it a `Coord` value. Only locations within the bounds of the grid are visited. The center location is included among the visited locations. For example, a radius of 1.0 includes only the center location plus four neighboring locations. A radius of 1.5 includes the center plus all the eight-way neighbors. The radius can be arbitrarily large but large radii require lots of CPU cycles.

<a name="InstallingTheCode"></a>

## Installing the code

---

Clone or copy the directory structure to a location of your choice.

<a name="BuildingTheExecutable"></a>

## Building the executable

---

<a name="SystemRequirements"></a>

### System requirements

This Rust implementation requires:

-  **Rust toolchain**: Rust 1.70 or later (install via [rustup](https://rustup.rs/))
-  **FFmpeg**: For video generation (optional but recommended)
   -  macOS: `brew install ffmpeg`
   -  Ubuntu/Debian: `sudo apt-get install ffmpeg`
   -  Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)
-  **Gnuplot**: For graph generation (optional)
   -  macOS: `brew install gnuplot`
   -  Ubuntu/Debian: `sudo apt-get install gnuplot`
-  **Python 3.4+**: For test scripts and tools
-  **python-igraph**: For neural network visualization (optional)
   -  `pip3 install python-igraph`

<a name="Compiling"></a>

### Compiling

The Rust implementation uses Cargo for building. To build the project:

#### Debug build

```sh
cargo build
```

The executable will be at `target/debug/biosimrust`.

#### Release build (recommended)

```sh
cargo build --release
```

The executable will be at `target/release/biosimrust`. This is optimized for performance.

#### Running tests

```sh
cargo test
```

To run with output:

```sh
cargo test -- --nocapture
```

#### Running specific tests

```sh
cargo test --test unit_tests
```

<a name="Execution"></a>

## Execution

---

Test everything is working by executing the release executable with the default config file (`biosimrust.ini`):

```sh
cargo run --release
```

Or if already built:

```sh
./target/release/biosimrust
```

You should see output something like:

```
Gen 1, 2290 survivors
```

If this works, edit the config file (`biosimrust.ini`) for the parameters you want for the simulation run and execute the executable. Optionally specify the name of the config file as the first command line argument:

```sh
cargo run --release -- biosimrust.ini
```

Or:

```sh
./target/release/biosimrust biosimrust.ini
```

### Real-time visualization

To enable real-time visualization, set `displayenabled = true` in the config file. A window will open showing the simulation as it runs. You can also enable challenge area highlighting with `displaychallengearea = true` to see the survival challenge boundaries overlaid on the simulation.

### Video generation

Videos are automatically generated at the end of each generation (if `savevideo = true`). The video framerate can be adjusted with the `videoframerate` parameter (default: 25 fps). Higher values create faster/shorter videos, lower values create slower/longer videos.

<a name="Testing"></a>

## Testing

---

The project includes both Rust unit tests and a Python-based integration test framework.

### Rust unit tests

Run all unit tests:

```sh
cargo test
```

The unit tests verify:

-  Basic type conversions and operations
-  Neural network wiring from genomes
-  Grid neighborhood visiting

### Python test framework

The Python test framework in the `tests/` directory automatically detects whether you're using the Rust or C++ implementation and runs the appropriate binary.

To run tests:

```sh
cd tests
python3 testapp.py --test microtest
```

The test framework will:

-  Detect the Rust project (via `Cargo.toml`)
-  Use `cargo run --release` to execute the Rust binary
-  Use the appropriate config file (`biosimrust.ini` for Rust)
-  Validate simulation results

See `tests/README.md` for more information about the test framework.

<a name="ToolsDirectory"></a>

## Tools directory

---

The tools directory (`tools/`) contains analysis and visualization scripts:

-  `tools/graphlog.gp` takes the generated log file `logs/epoch-log.txt` and generates a graphic plot of the simulation run in `images/log.png`. You may need to adjust the directory paths in `graphlog.gp` for your environment. `graphlog.gp` can be invoked manually, or if the option `updategraphlog` is set to `true` in the simulation config file, the simulator will try to invoke `tools/graphlog.gp` automatically during the simulation run. Also see the parameter named `updategraphlogstride` in the config file.

-  `tools/graph-nnet.py` takes a text file (hardcoded name "net.txt") and generates a neural net connection diagram using igraph. The file `net.txt` contains an encoded form of one genome, and must be the same format as the files generated by `display_sample_genomes()` in `src/analysis.rs` which is called by `simulator()` in `src/simulator.rs`. The genome output is printed to stdout automatically if the parameter named `displaysamplegenomes` is set to nonzero in the config file. An individual genome can be copied from that output stream and renamed "net.txt" in order to run `graph-nnet.py`.

Note: The tools are shared between the Rust and C++ implementations and work with output from both versions.

<a name="RustVsCpp"></a>

## Rust vs C++ Implementation

---

### Key differences

-  **Memory safety**: Rust's ownership system eliminates many classes of bugs (use-after-free, data races, etc.)
-  **Parallelization**: Uses `rayon` instead of OpenMP for thread-safe parallel execution
-  **Image/video generation**: Uses the `image` crate and FFmpeg instead of CImg/OpenCV
-  **Real-time display**: Uses `minifb` for cross-platform windowing
-  **Config file**: Uses `biosimrust.ini`

### File structure

-  **Rust source**: `src/*.rs` files
-  **Shared tools**: `tools/` directory (works with both implementations)
-  **Shared tests**: `tests/` directory (automatically detects Rust vs C++)

### Performance

The Rust implementation should have similar or better performance than the C++ version due to:

-  Zero-cost abstractions
-  Efficient parallelization with `rayon`
-  Optimized release builds with `cargo build --release`

### Feature parity

The Rust implementation maintains full feature parity with the original C++ version:

-  All 19 challenge types
-  All sensors and actions
-  Neural network evolution
-  Video generation
-  Real-time visualization
-  Parameter hot-reloading
-  All analysis and reporting features

### Additional features

The Rust implementation adds:

-  Adjustable challenge area visualization with opacity control
-  Configurable video framerate
-  Improved error handling and validation
-  Better cross-platform compatibility

## Project structure

```
biosimrust/
├── src/                    # Rust source files
│   ├── *.rs               # Rust modules
│   └── lib.rs             # Library root
├── tests/                  # Test framework (works with both Rust and C++)
│   ├── unit_tests.rs      # Rust unit tests
│   └── testapp.py         # Python integration tests (if present)
├── Cargo.toml             # Rust project configuration
├── biosimrust.ini         # Rust config file
└── README.md              # This file
```

## Contributing

This is a refactored version of the original C++ biosim4 project, renamed to BiosimRust. When contributing:

-  Follow Rust best practices and idioms
-  Maintain compatibility with the original behavior where possible
-  Add tests for new features

## License

See the LICENSE file for license information.
