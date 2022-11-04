
/// This struct represents the entire Cellular Automaton. 
pub struct WorldState {
    size: u16,
}

impl WorldState {
    
    /// Create a new **WorldState** with a defined size.
    /// The size provided is used for the width and the height.
    pub fn new(w_size: u16) -> WorldState {
        WorldState {
            size: w_size,
        }
    }
}
