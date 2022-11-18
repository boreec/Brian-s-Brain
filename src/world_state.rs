use crate::graphics::vulkan::Vertex;

use rand::prelude::SliceRandom;
use rand::thread_rng;

use std::fmt;

const ALIVE_COLOR: [f32; 3] = [1.0, 0.0, 0.0];
const DYING_COLOR: [f32; 3] = [0.5, 0.0, 0.0];

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

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            CellState::On => { 'O' }
            CellState::Off => {'.'}
            CellState::Dying => {'X'}
        };
        
        write!(f, "{c}")
    }    
}

/// This struct represents the entire Cellular Automaton. 
pub struct WorldState {
    size: u16,
    world: Vec<CellState>,
}

impl fmt::Display for WorldState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::from("");
        for (i, item) in self.world.iter().enumerate(){
            s.push_str(&item.to_string());
            if (i + 1) % self.size as usize == 0 {
                s.push('\n');
            }
        }
        write!(f, "{s}")
    }    
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
            self.world = vec![CellState::On; self.world.len()];
            return;
        }
        let mut cell_indexes: Vec<_> = (0..self.world.len()).collect();
        let cell_amount = (on_rate * (self.world.len() as f64)) as usize;
        
        cell_indexes.shuffle(&mut thread_rng());
        for item in cell_indexes.iter_mut().take(cell_amount) {
            self.world[*item as usize] = CellState::On;
        }
    }

    /// Advance the world to its next state.
    /// A cell **On** is turned into **Dying**.
    /// A cell **Dying** is turned into **Off**.
    /// A cell **Off** is turned into **On** if two of its neighbours
    /// are also in **On** State.
    pub fn next(&mut self) {
        let mut new_dyings: Vec<_> = vec![];
        let mut new_on: Vec<_> = vec![];
        let mut new_off: Vec<_> = vec![];
        
        for (i, item) in self.world.iter().enumerate() {
            match item {
                CellState::On => { new_dyings.push(i); }
                CellState::Off => {
                    let neighbours = self.get_neighbours(
                            (i % self.size as usize) as u16,
                            (i / self.size as usize) as u16
                    );
                    let alives = {
                        let mut sum = 0;
                        for tuples in neighbours {
                            let cell_idx = (tuples.0 * self.size + tuples.1) as usize;
                            let cell_state = &self.world[cell_idx];
                            if *cell_state == CellState::On {
                                sum += 1;
                            } 
                        }
                        sum
                    };
                    if alives == 2 {
                        new_on.push(i);
                    }
                }
                CellState::Dying => { new_off.push(i); }
            }
        }
        // update the world
        for (_, item) in new_dyings.iter().enumerate() { 
            self.world[*item] = CellState::Dying; 
        }
        for (_, item) in new_off.iter().enumerate() { 
            self.world[*item] = CellState::Off; 
        }
        for (_, item) in new_on.iter().enumerate() { 
            self.world[*item] = CellState::On; 
        }
    }
    
    /// Return the neighbours of a cell designated by the coordinates `x` and `y`.
    /// The neighbours are returned as a vector of tuples (x_i, y_i). If the given
    /// coordinates are outside the world, the returned vector is empty.
    fn get_neighbours(&self, x: u16, y: u16) -> Vec<(u16, u16)> {
        // general case: 8 neighbours
        if x > 0 && x < self.size - 1 && y > 0 && y < self.size - 1 {
            vec![
                (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
                (x - 1, y), (x + 1, y),
                (x - 1, y + 1), (x, y + 1), (x + 1, y + 1)
            ]
        }
        // top left corner: 3 neighbours
        else if x == 0 && y == 0 {
            vec![(x, y + 1), (x + 1, y + 1), (x + 1, y)]
        } 
        // top right corner: 3 neighbours
        else if x == self.size - 1 && y == 0 {
            vec![(x - 1, y), (x - 1, y + 1), (x, y + 1)]
        }
        // bottom left corner: 3 neighbours
        else if x == 0 && y == self.size - 1 {
            vec![(x, y - 1), (x + 1, y - 1), (x + 1, y)]
        }
        // bottom right corner: 3 neighbours
        else if x == self.size - 1 && y == self.size - 1 {
            vec![(x - 1, y), (x - 1, y - 1), (x, y - 1)]
        }
        // top edge: 5 neighbours
        else if y == 0 {
            vec![(x - 1, y), (x + 1, y), (x - 1, y + 1), (x, y + 1), (x + 1, y + 1)]
        }
        // bottom edge: 5 neighbours
        else if y == self.size - 1 {
            vec![(x - 1, y), (x + 1, y), (x - 1, y - 1), (x, y - 1), (x + 1, y - 1)]
        }
        // left edge: 5 neighbours
        else if x == 0 {
            vec![(x, y - 1), (x, y + 1), (x + 1, y - 1), (x + 1, y), (x + 1, y + 1)]
        }
        // right edge: 5 neighbours
        else if x == self.size - 1 {
            vec![(x, y - 1), (x, y + 1), (x - 1, y - 1), (x - 1, y), (x - 1, y + 1)]
        }         
        // outside the range of the world: 0 neighbours
        else {
            vec![]
        }
    }   
    
    /// Return vertices of the cells with `CellState::On` or `CellState::Dying`.
    /// Moreover, each cell is represented by 6 vertices (2 triangles).
    pub fn as_vertices(&self) -> Vec<Vertex> {
        let mut updated_cells: Vec<Vertex> = vec![];
        
        let cell_w = 2.0 / self.size as f32;
        let cell_h = 2.0 / self.size as f32;
        for (i, item) in self.world.iter().enumerate() {
            let cell_x = (i % self.size as usize) as f32;
            let cell_y = (i / self.size as usize) as f32;
            
            // left triangle : ◺
            let (x1, y1) = (-1.0 + cell_w * cell_x, -1.0 + cell_h * cell_y);
            let (x2, y2) = (-1.0 + cell_w * cell_x, -1.0 + cell_h * (cell_y + 1.0));
            let (x3, y3) = (-1.0 + cell_w * (cell_x + 1.0), -1.0 + cell_h * (cell_y + 1.0));
            // right triangle : ◹ 
            let (x4, y4) = (x1, y1);
            let (x5, y5) = (-1.0 + cell_w * (cell_x + 1.0), -1.0 + cell_h * cell_y);
            let (x6, y6) = (x3, y3);
            
            match item {
                CellState::On => {
                    let mut cell_vertices = vec![
                        Vertex { position: [x1, y1], color: ALIVE_COLOR},
                        Vertex { position: [x2, y2], color: ALIVE_COLOR},  
                        Vertex { position: [x3, y3], color: ALIVE_COLOR},  
                        Vertex { position: [x4, y4], color: ALIVE_COLOR},  
                        Vertex { position: [x5, y5], color: ALIVE_COLOR},  
                        Vertex { position: [x6, y6], color: ALIVE_COLOR},  
                    ];
                    updated_cells.append(&mut cell_vertices);
                }
                CellState::Dying => {
                    let mut cell_vertices = vec![
                        Vertex { position: [x1, y1], color: DYING_COLOR},
                        Vertex { position: [x2, y2], color: DYING_COLOR},  
                        Vertex { position: [x3, y3], color: DYING_COLOR},  
                        Vertex { position: [x4, y4], color: DYING_COLOR},  
                        Vertex { position: [x5, y5], color: DYING_COLOR},  
                        Vertex { position: [x6, y6], color: DYING_COLOR},  
                    ];
                    updated_cells.append(&mut cell_vertices);
                }
                CellState::Off => {}
            }
        }    
        updated_cells
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    fn count(ws: &WorldState, c: CellState) -> usize {
        ws.to_string().matches(&c.to_string()).count()
    }
    
    #[test]
    fn test_randomize_for_rate_equal_one() {
        let mut ws = WorldState::new(100);
        ws.randomize(1.0);
        assert_eq!(count(&ws, CellState::On), 10_000);
    }

    #[test]
    fn test_randomize_for_rate_equal_zero() {
        let mut ws = WorldState::new(100);
        ws.randomize(0.0);
        assert_eq!(count(&ws, CellState::Off), 10_000);    
    }
    
    #[test]
    fn test_randomize_for_rate_equal_one_point_five() {
        let mut ws = WorldState::new(100);
        ws.randomize(0.5);
        assert_eq!(count(&ws, CellState::Off), 5_000);    
        assert_eq!(count(&ws, CellState::On), 5_000);    
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
    
    #[test]
    fn test_get_neighbours_bottom_left_corner() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(0, 9);
        assert_eq!(neighbours.len(), 3);
        assert!(neighbours.contains(&(0,8)));
        assert!(neighbours.contains(&(1,8)));
        assert!(neighbours.contains(&(1,9)));
    }    
    
    #[test]
    fn test_get_neighbours_bottom_right_corner() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(9, 9);
        assert_eq!(neighbours.len(), 3);
        assert!(neighbours.contains(&(9,8)));
        assert!(neighbours.contains(&(8,8)));
        assert!(neighbours.contains(&(8,9)));
    }
    
    #[test]
    fn test_get_neighbours_top_edge() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(4, 0);
        assert_eq!(neighbours.len(), 5);
        assert!(neighbours.contains(&(3,0)));
        assert!(neighbours.contains(&(5,0)));
        assert!(neighbours.contains(&(4,1)));
        assert!(neighbours.contains(&(3,1)));
        assert!(neighbours.contains(&(5,1)));
    }    
    
    #[test]
    fn test_get_neighbours_bottom_edge() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(4, 9);
        assert_eq!(neighbours.len(), 5);
        assert!(neighbours.contains(&(3,9)));
        assert!(neighbours.contains(&(5,9)));
        assert!(neighbours.contains(&(4,8)));
        assert!(neighbours.contains(&(3,8)));
        assert!(neighbours.contains(&(5,8)));
    }
    
    #[test]
    fn test_get_neighbours_left_edge() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(0, 5);
        assert_eq!(neighbours.len(), 5);
        assert!(neighbours.contains(&(0,4)));
        assert!(neighbours.contains(&(0,6)));
        assert!(neighbours.contains(&(1,6)));
        assert!(neighbours.contains(&(1,4)));
        assert!(neighbours.contains(&(1,5)));
    }    

    #[test]
    fn test_get_neighbours_right_edge() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(9, 5);
        assert_eq!(neighbours.len(), 5);
        assert!(neighbours.contains(&(9,4)));
        assert!(neighbours.contains(&(9,6)));
        assert!(neighbours.contains(&(8,6)));
        assert!(neighbours.contains(&(8,4)));
        assert!(neighbours.contains(&(8,5)));
    }    

    #[test]
    fn test_get_neighbours_general_case() {
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(5, 5);
        assert_eq!(neighbours.len(), 8);
        assert!(neighbours.contains(&(4,4)));
        assert!(neighbours.contains(&(5,4)));
        assert!(neighbours.contains(&(6,4)));
        assert!(neighbours.contains(&(4,5)));
        assert!(neighbours.contains(&(6,5)));
        assert!(neighbours.contains(&(4,6)));
        assert!(neighbours.contains(&(5,6)));
        assert!(neighbours.contains(&(6,6)));
    }
    
    #[test]
    fn test_get_neighbours_outside(){
        let ws = WorldState::new(10);
        let neighbours = ws.get_neighbours(10,10);
        assert_eq!(neighbours.len(), 0);
    }
    
    #[test]
    fn test_as_vertices_for_one_cell_world(){
        // declare a world with just one cell.
        let mut ws = WorldState::new(1);
        // set the cell to On state.
        ws.randomize(1.0);
        let cells = ws.as_vertices();
        assert_eq!(cells.len(), 6);
        
        // advance to next iteration: the cell must be in dying mode.
        ws.next();
        let cells = ws.as_vertices();
        assert_eq!(cells.len(), 6);
        
        // advance to next iteration: the cell must be dead.
        ws.next();
        let cells = ws.as_vertices();
        assert_eq!(cells.len(), 0);
    }
    
    #[test]
    fn test_as_vertices_good_coordinates_for_one_cell_world() {
        let mut ws = WorldState::new(1);
        ws.randomize(1.0);
        let cells = ws.as_vertices();
        assert!(cells.contains( &Vertex { position: [-1.0, -1.0], color: ALIVE_COLOR }));
        assert!(cells.contains( &Vertex { position: [-1.0, 1.0], color: ALIVE_COLOR }));
        assert!(cells.contains( &Vertex { position: [1.0, -1.0], color: ALIVE_COLOR }));
        assert!(cells.contains( &Vertex { position: [1.0, 1.0], color: ALIVE_COLOR }));
        ws.next();
        let cells = ws.as_vertices();
        assert!(cells.contains( &Vertex { position: [-1.0, -1.0], color: DYING_COLOR }));
        assert!(cells.contains( &Vertex { position: [-1.0, 1.0], color: DYING_COLOR }));
        assert!(cells.contains( &Vertex { position: [1.0, -1.0], color: DYING_COLOR }));
        assert!(cells.contains( &Vertex { position: [1.0, 1.0], color: DYING_COLOR }));
    }
}
