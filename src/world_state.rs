use array2d::Array2D;

/// The three states a cell can take.
/// Each cell is considered to have 8 neighbors (the Moore neighborhood).
/// In each time step, a cell turns on if it was **Off** but had exactly two neighbors
/// that were on. All cells that were **On** go into the **Dying** state, which is not
/// counted as an **On** cell in the neighbor count, and prevents any cell from being
/// born there. Cells that were in the **Dying** state go into the **Off** state. 
#[derive(Clone, PartialEq)]
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
    
    /// Create a new **WorldState** with a defined `size`.
    ///
    /// The `size` provided is used for the world's width and height.
    pub fn new(size: u16) -> WorldState {
        WorldState {
            size,
            world: Array2D::filled_with(CellState::Off, size as usize, size as usize),
        }
    }
    
    /// Initialize the world with a certain amount of **CellState::On**.
    /// 
    /// `on_rate` corresponds to the percentage of cells in the world to
    /// set their state to **CellState::On**. `on_rate` is expected to be
    /// between 0 and 1. Any value outside that range will cause a panic.
    pub fn randomize(&self, on_rate: f64) {
        
    } 
}
