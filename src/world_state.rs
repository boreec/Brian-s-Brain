enum CellState {
    On,
    Dying,
    Off,
}

/// This struct represents the entire Cellular Automaton. 
pub struct WorldState {
    size: u16,
}

impl WorldState {
    
    /// Create a new **WorldState** with a defined size.
    /// The size provided is used for the width and the height.
    pub fn new(size: u16) -> WorldState {
        WorldState {
            size,
        }
    }
}
