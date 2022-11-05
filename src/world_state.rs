use array2d::Array2D;

#[derive(Clone)]
enum CellState {
    On,
    Dying,
    Off,
}

/// This struct represents the entire Cellular Automaton. 
pub struct WorldState {
    size: u16,
    world: Array2D<CellState>,
}

impl WorldState {
    
    /// Create a new **WorldState** with a defined size.
    /// The size provided is used for the width and the height.
    pub fn new(size: u16) -> WorldState {
        WorldState {
            size,
            world: Array2D::filled_with(CellState::Off, size as usize, size as usize),
        }
    }
}
