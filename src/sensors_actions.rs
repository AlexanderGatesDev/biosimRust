// sensors_actions.rs
// Defines which sensor input neurons and which action output neurons
// are compiled into the simulator

pub const SENSOR_MIN: f32 = 0.0;
pub const SENSOR_MAX: f32 = 1.0;
pub const SENSOR_RANGE: f32 = SENSOR_MAX - SENSOR_MIN;

pub const NEURON_MIN: f32 = -1.0;
pub const NEURON_MAX: f32 = 1.0;
pub const NEURON_RANGE: f32 = NEURON_MAX - NEURON_MIN;

pub const ACTION_MIN: f32 = 0.0;
pub const ACTION_MAX: f32 = 1.0;
pub const ACTION_RANGE: f32 = ACTION_MAX - ACTION_MIN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Sensor {
    LocX = 0,              // I distance from left edge
    LocY,                   // I distance from bottom
    BoundaryDistX,          // I X distance to nearest edge of world
    BoundaryDist,           // I distance to nearest edge of world
    BoundaryDistY,          // I Y distance to nearest edge of world
    GeneticSimFwd,          // I genetic similarity forward
    LastMoveDirX,           // I +- amount of X movement in last movement
    LastMoveDirY,           // I +- amount of Y movement in last movement
    LongprobePopFwd,        // W long look for population forward
    LongprobeBarFwd,        // W long look for barriers forward
    Population,             // W population density in neighborhood
    PopulationFwd,          // W population density in the forward-reverse axis
    PopulationLr,           // W population density in the left-right axis
    Osc1,                   // I oscillator +-value
    Age,                    // I
    BarrierFwd,             // W neighborhood barrier distance forward-reverse axis
    BarrierLr,              // W neighborhood barrier distance left-right axis
    Random,                 //   random sensor value, uniform distribution
    Signal0,                // W strength of signal0 in neighborhood
    Signal0Fwd,              // W strength of signal0 in the forward-reverse axis
    Signal0Lr,               // W strength of signal0 in the left-right axis
    NumSenses,              // <<------------------ END OF ACTIVE SENSES MARKER
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Action {
    MoveX = 0,              // W +- X component of movement
    MoveY,                  // W +- Y component of movement
    MoveForward,            // W continue last direction
    MoveRl,                 // W +- component of movement
    MoveRandom,              // W
    SetOscillatorPeriod,    // I
    SetLongprobeDist,       // I
    SetResponsiveness,       // I
    EmitSignal0,            // W
    MoveEast,               // W
    MoveWest,               // W
    MoveNorth,              // W
    MoveSouth,              // W
    MoveLeft,               // W
    MoveRight,              // W
    MoveReverse,            // W
    NumActions,             // <<----------------- END OF ACTIVE ACTIONS MARKER
    KillForward,            // W
}

impl Sensor {
    pub fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl Action {
    pub fn as_usize(&self) -> usize {
        *self as usize
    }
}

pub fn sensor_name(sensor: Sensor) -> &'static str {
    match sensor {
        Sensor::LocX => "LOC_X",
        Sensor::LocY => "LOC_Y",
        Sensor::BoundaryDistX => "BOUNDARY_DIST_X",
        Sensor::BoundaryDist => "BOUNDARY_DIST",
        Sensor::BoundaryDistY => "BOUNDARY_DIST_Y",
        Sensor::GeneticSimFwd => "GENETIC_SIM_FWD",
        Sensor::LastMoveDirX => "LAST_MOVE_DIR_X",
        Sensor::LastMoveDirY => "LAST_MOVE_DIR_Y",
        Sensor::LongprobePopFwd => "LONGPROBE_POP_FWD",
        Sensor::LongprobeBarFwd => "LONGPROBE_BAR_FWD",
        Sensor::Population => "POPULATION",
        Sensor::PopulationFwd => "POPULATION_FWD",
        Sensor::PopulationLr => "POPULATION_LR",
        Sensor::Osc1 => "OSC1",
        Sensor::Age => "AGE",
        Sensor::BarrierFwd => "BARRIER_FWD",
        Sensor::BarrierLr => "BARRIER_LR",
        Sensor::Random => "RANDOM",
        Sensor::Signal0 => "SIGNAL0",
        Sensor::Signal0Fwd => "SIGNAL0_FWD",
        Sensor::Signal0Lr => "SIGNAL0_LR",
        Sensor::NumSenses => "NUM_SENSES",
    }
}

pub fn action_name(action: Action) -> &'static str {
    match action {
        Action::MoveX => "MOVE_X",
        Action::MoveY => "MOVE_Y",
        Action::MoveForward => "MOVE_FORWARD",
        Action::MoveRl => "MOVE_RL",
        Action::MoveRandom => "MOVE_RANDOM",
        Action::SetOscillatorPeriod => "SET_OSCILLATOR_PERIOD",
        Action::SetLongprobeDist => "SET_LONGPROBE_DIST",
        Action::SetResponsiveness => "SET_RESPONSIVENESS",
        Action::EmitSignal0 => "EMIT_SIGNAL0",
        Action::MoveEast => "MOVE_EAST",
        Action::MoveWest => "MOVE_WEST",
        Action::MoveNorth => "MOVE_NORTH",
        Action::MoveSouth => "MOVE_SOUTH",
        Action::MoveLeft => "MOVE_LEFT",
        Action::MoveRight => "MOVE_RIGHT",
        Action::MoveReverse => "MOVE_REVERSE",
        Action::NumActions => "NUM_ACTIONS",
        Action::KillForward => "KILL_FORWARD",
    }
}

pub const NUM_SENSES: usize = Sensor::NumSenses as usize;
pub const NUM_ACTIONS: usize = Action::NumActions as usize;

pub fn print_sensors_actions() {
    println!("Sensors:");
    for i in 0..NUM_SENSES {
        println!("  {}: {}", i, sensor_name(unsafe { std::mem::transmute(i as u8) }));
    }
    println!("Actions:");
    for i in 0..NUM_ACTIONS {
        println!("  {}: {}", i, action_name(unsafe { std::mem::transmute(i as u8) }));
    }
}

