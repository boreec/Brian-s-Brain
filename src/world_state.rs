use rand::prelude::SliceRandom;
use rand::thread_rng;

/// The three states a cell can take.
/// Each cell is considered to have 8 neighbors (the Moore neighborhood).
/// In each time step, a cell turns on if it was **Off** but had exactly two neighbors
/// that were on. All cells that were **On** go into the **Dying** state, which is not
/// counted as an **On** cell in the neighbor count, and prevents any cell from being
/// born there. Cells that were in the **Dying** state go into the **Off** state. 
#[derive(Clone, Debug, PartialEq, Eq)]
enum CellState {
    On,
    Dying,
    Off,
}

/// This struct represents the entire Cellular Automaton. 
pub struct WorldState {
    size: u16,
    world: Vec<CellState>,
}

impl WorldState {
    
    /// Create a new **WorldState** with a defined `size`.
    ///
    /// The `size` provided is used for the world's width and height.
    pub fn new(size: u16) -> WorldState {
        WorldState {
            size,
            world: vec![CellState::Off; size.pow(2).into()],
        }
    }
    
    /// Initialize the world with a certain amount of **CellState::On**.
    /// 
    /// `on_rate` corresponds to the percentage of cells in the world to
    /// set their state to **CellState::On**. `on_rate` is expected to be
    /// between 0 and 1. Any value outside that range will cause a panic.
    pub fn randomize(&mut self, on_rate: f64) {
        if on_rate == 1.0 {
            self.world = vec![CellState::On; self.size.pow(2).into()];
            return;
        }
        let mut cell_indexes: Vec<_> = (0..self.size).collect();
        cell_indexes.shuffle(&mut thread_rng());
        
        let cell_amount = (on_rate * (self.world.len() as f64)) as usize;
        for i in 0..cell_amount {
            self.world[i] = CellState::On;
        }
    }

    fn get_neighbours(&self, x: u16, y: u16) -> Vec<(u16, u16)> {
        // top left corner
        if x == 0 && y == 0 {
            return vec![(x, y + 1), (x + 1, y + 1), (x + 1, y)];
        } 
        // top right corner
        else if x == self.size - 1 && y == 0 {
            return vec![(x - 1, y), (x - 1, y + 1), (x, y + 1)];
        }
        return vec![];
    }   
    
    fn get_cell(&self, row: u16, col: u16) -> Option<&CellState> {
        if row * col + col > self.size.pow(2) {
            return None;
        }
        
        Some(&self.world[(row * col + col) as usize])
    }
    
    fn get_size(&self) -> u16 {
        self.size
    }
    
    fn count(&self, state: CellState) -> usize {
        self.world.iter().filter(|&c| *c == state).count()
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn test_randomize_for_rate_equal_one() {
        let mut ws = WorldState::new(100);
        ws.randomize(1.0);
        assert_eq!(ws.count(CellState::On), 10_000);
    }

    #[test]
    fn test_randomize_for_rate_equal_zero() {
        let mut ws = WorldState::new(100);
        ws.randomize(0.0);
        assert_eq!(ws.count(CellState::Off), 10_000);    
    }
    
    #[test]
    fn test_randomize_for_rate_equal_one_point_five() {
        let mut ws = WorldState::new(100);
        ws.randomize(0.5);
        assert_eq!(ws.count(CellState::Off), 5_000);    
        assert_eq!(ws.count(CellState::On), 5_000);    
    }
    
    #[test]
    fn test_get_neighbours_top_left_corner() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(0, 0);
        assert_eq!(neighbours.len(), 3);
        assert!(neighbours.contains(&(0,1)));
        assert!(neighbours.contains(&(1,1)));
        assert!(neighbours.contains(&(1,0)));
    }    

    #[test]
    fn test_get_neighbours_top_right_corner() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(9, 0);
        assert_eq!(neighbours.len(), 3);
        assert!(neighbours.contains(&(8,0)));
        assert!(neighbours.contains(&(8,1)));
        assert!(neighbours.contains(&(9,1)));
    }    
}
