use crate::graphics::vulkan::Vertex;

use rand::prelude::SliceRandom;
use rand::thread_rng;

use std::fmt;

/// The color used to represent on a GUI the cells alive.
/// The content is an array representing the RGB values.
const ALIVE_COLOR: [f32; 3] = [1.0, 0.0, 0.0];

/// The color used to represent on a GUI the cells dying.
/// The content is an array representing the RGB values.
const DYING_COLOR: [f32; 3] = [0.5, 0.0, 0.0];

/// The three states a cell can take.
/// Each cell is considered to have 8 neighbors (the Moore neighborhood).
/// In each time step, a cell turns on if it was **Off** but had exactly two neighbors
/// that were on. All cells that were **On** go into the **Dying** state, which is not
/// counted as an **On** cell in the neighbor count, and prevents any cell from being
/// born there. Cells that were in the **Dying** state go into the **Off** state. 
#[derive(Clone, Debug, PartialEq, Eq)]
enum CellState {
    Alive,
    Dying,
    Dead,
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            CellState::Alive => { 'O' }
            CellState::Dead => {'.'}
            CellState::Dying => {'X'}
        };
        
        write!(f, "{c}")
    }    
}

/// This struct represents the entire Cellular Automaton. 
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldState {
    
    /// The size of the world representing the Cellular Automaton.
    /// This value is *one side* of the world, and thus the *real* size
    /// is this value squared (because the world is 2D).
    size: u16,
    
    /// The actual representation of the Cellular Automaton at a given time.
    /// It consists of a 1D vector of `CellState` values.
    world: Vec<CellState>,
    
    neighbours: Vec<Vec<u16>>,
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
            world: vec![CellState::Dead; size.pow(2).into()],
            neighbours: Self::precompute_neighbours(size),
        }
    }
    
    /// Compute every neighbours for each cell of the CA.
    fn precompute_neighbours(size: u16) -> Vec<Vec<u16>> {
        let mut neighbours: Vec<Vec<u16>> = vec![];
        for i in 0..size.pow(2){
            let x = i % size;
            let y = i / size;
            // general case: 8 neighbours
            if x > 0 && x < size - 1 && y > 0 && y < size - 1 {
                neighbours.push(vec![
                    (y - 1) * size + x - 1,
                    (y - 1) * size + x,
                    (y - 1) * size + x + 1,
                    y * size + x - 1,
                    y * size + x + 1,
                    (y + 1) * size + x - 1,
                    (y + 1) * size + x,
                    (y + 1) * size + x + 1
                ]);
            }
            // top left corner: 3 neighbours
            else if x == 0 && y == 0 {
                neighbours.push(vec![1, size, size + 1])
            }  
            // top right corner: 3 neighbours
            else if x == size - 1 && y == 0 {
                neighbours.push(vec![x - 1, size + x - 1, size + x]);
            }
            // bottom left corner: 3 neighbours
            else if x == 0 && y == size - 1 {
                neighbours.push(vec![(y - 1) * size, (y - 1) * size + 1, y * size + 1]);
            }
            // bottom right corner: 3 neighbours
            else if x == size - 1 && y == size - 1 {
                neighbours.push(vec![y * size + x - 1,(y - 1) * size + x - 1, (y - 1) * size + x]);
            }
            // top edge: 5 neighbours
            else if y == 0 {
                neighbours.push(vec![x - 1, x + 1, size + x - 1, size + x, size + x + 1])
            }
            // bottom edge: 5 neighbours
            else if y == size - 1 {
                neighbours.push(vec![
                    (y - 1) * size + x - 1,
                    (y - 1) * size + x,
                    (y - 1) * size + x + 1,
                    y * size + x - 1,
                    y * size + x + 1,
                ]);
            }
            //left edge: 5 neighbours
            else if x == 0 && y < size {
                neighbours.push(vec![
                    (y - 1) * size,
                    (y - 1) * size + 1,
                    y * size + 1,
                    (y + 1) * size,
                    (y + 1) * size + 1    
                ]);
            }
            // right edge: 5 neighbours
            else if x == size - 1 {
                neighbours.push(vec![
                    (y - 1) * size + x - 1,
                    (y - 1) * size + x,
                    y * size + x - 1,
                    (y + 1) * size + x - 1,
                    (y + 1) * size + x,
                ]);
            }         
            // outside the range of the world: 0 neighbours
            else {
                neighbours.push(vec![]);
            }
        }
        neighbours
    }   

    /// Initialize the world with a certain amount of **CellState::On**.
    /// 
    /// `on_rate` corresponds to the percentage of cells in the world to
    /// set their state to **CellState::On**. `on_rate` is expected to be
    /// between 0 and 1. Any value outside that range will cause a panic.
    pub fn randomize(&mut self, on_rate: f64) {
        if on_rate == 1.0 {
            self.world = vec![CellState::Alive; self.world.len()];
            return;
        }
        let mut cell_indexes: Vec<_> = (0..self.world.len()).collect();
        let cell_amount = (on_rate * (self.world.len() as f64)) as usize;
        
        cell_indexes.shuffle(&mut thread_rng());
        for item in cell_indexes.iter_mut().take(cell_amount) {
            self.world[*item as usize] = CellState::Alive;
        }
    }

    /// Advance the world to its next state.
    /// A cell **Alive** is turned into **Dying**.
    /// A cell **Dying** is turned into **Dead**.
    /// A cell **Dead** is turned into **Alive** if two of its neighbours
    /// are also in **Alive** State.
    pub fn next(&mut self) {
        let mut new_dying: Vec<_> = vec![];
        let mut new_alive: Vec<_> = vec![];
        let mut new_dead: Vec<_> = vec![];
        
        for i in 0..self.world.len() {
            match self.world[i] {
                CellState::Alive => { new_dying.push(i); }
                CellState::Dead => {
                    let alives = self.neighbours[i]
                        .iter()
                        .filter(|&n| self.world[*n as usize] == CellState::Alive)
                        .count();
                    
                    if alives == 2 {
                        new_alive.push(i);
                    }
                }
                CellState::Dying => { new_dead.push(i); }
            }
        }
        // update the world
        for item in new_dying { 
            self.world[item] = CellState::Dying; 
        }
        for item in new_dead { 
            self.world[item] = CellState::Dead; 
        }
        for item in new_alive { 
            self.world[item] = CellState::Alive; 
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
                CellState::Alive => {
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
                CellState::Dead => {}
            }
        }    
        updated_cells
    }
    
    /// Initialize a world 14x14 with 5x3-period oscillators.
    /// Example made by **boreec**.
    pub fn example1() -> WorldState {
        let mut ws = WorldState::new(14);
        ws.spawn_osc3(0, 0);
        ws.spawn_osc3(10, 10);
        ws.spawn_osc3(0, 10);
        ws.spawn_osc3(10, 0);
        ws.spawn_osc3(5, 5);
        ws
    }
    
    /// Initialize a world 100x100 with many gliders creating
    /// a breeder. Example made by **Wojowu** on `conwaylife.com`.
    pub fn example2() ->  WorldState {
        let mut ws = WorldState::new(100);
        ws.spawn_glider4_downward(42, 0);
        ws.spawn_glider4_downward(30, 18);
        ws.spawn_glider4_downward(30, 22);
        ws.spawn_glider4_downward(13, 42);
        ws.spawn_glider4_leftward(23, 57);
        ws.spawn_glider4_leftward(19, 62);
        ws.spawn_glider4_upward(10, 68);
        ws.spawn_glider4_upward(24, 87);
        ws.spawn_glider4_upward(28, 93);
        ws
    }
    
    /// Initialize a world 100x100 with a wick.
    /// Example made by **The Turtle** on `conwaylife.com`.
    pub fn example3() -> WorldState {
        let mut ws = WorldState::new(100);
        ws.spawn_wick3(50, 50);
        ws
    }
    pub fn spawn_osc3(&mut self, x: usize, y: usize) {
        let dying_cells = [(x + 1, y + 1), (x + 2, y + 1), (x + 1, y + 2), (x + 2, y + 2)];
        let alive_cells = [(x, y + 1), (x + 2, y), (x + 1, y + 3), (x + 3, y + 2)];
        
        for i in alive_cells {
            self.world[i.0 * self.size as usize + i.1] = CellState::Alive;
        }
        for i in dying_cells {
            self.world[i.0 * self.size as usize + i.1] = CellState::Dying;
        }    
    }
    
    pub fn spawn_glider4_downward(&mut self, x: usize, y: usize){
        let dying_cells = [(x, y), (x + 1, y)];
        let alive_cells = [(x, y + 1), (x + 1, y + 1)];
        
        for i in alive_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Alive;
        }
        for i in dying_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Dying;
        }    
    }
    
    pub fn spawn_glider4_upward(&mut self, x: usize, y: usize){
        let alive_cells = [(x, y), (x + 1, y)];
        let dying_cells = [(x, y + 1), (x + 1, y + 1)];
        
        for i in alive_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Alive;
        }
        for i in dying_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Dying;
        }    
    }
    
    pub fn spawn_glider4_leftward(&mut self, x: usize, y: usize){
        let dying_cells = [(x + 1, y), (x + 1, y + 1)];
        let alive_cells = [(x, y), (x, y + 1)];
        
        for i in alive_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Alive;
        }
        for i in dying_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Dying;
        }    
    }
    
    pub fn spawn_wick3(&mut self, x: usize, y: usize) {
        let dying_cells = [(x + 1, y + 2)];
        let alive_cells = [
            (x, y), (x + 1, y), (x + 2, y),
            (x, y + 1), (x + 2, y + 1)
        ];
        
        for i in alive_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Alive;
        }
        for i in dying_cells {
            self.world[i.1 * self.size as usize + i.0] = CellState::Dying;
        }    
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
        assert_eq!(count(&ws, CellState::Alive), 10_000);
    }

    #[test]
    fn test_randomize_for_rate_equal_zero() {
        let mut ws = WorldState::new(100);
        ws.randomize(0.0);
        assert_eq!(count(&ws, CellState::Dead), 10_000);    
    }
    
    #[test]
    fn test_randomize_for_rate_equal_one_point_five() {
        let mut ws = WorldState::new(100);
        ws.randomize(0.5);
        assert_eq!(count(&ws, CellState::Dead), 5_000);    
        assert_eq!(count(&ws, CellState::Alive), 5_000);    
    }
    
    #[test]
    fn test_get_neighbours_top_left_corner() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[0], vec![1, 10, 11]);
    }    

    #[test]
    fn test_get_neighbours_top_right_corner() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[9], vec![8, 18, 19]);
    }    
    
    #[test]
    fn test_get_neighbours_bottom_left_corner() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[90], vec![80, 81, 91]);
    }    
    
    #[test]
    fn test_get_neighbours_bottom_right_corner() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[99], vec![88, 89, 98]);
    }
    
    #[test]
    fn test_get_neighbours_top_edge() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[4], vec![3, 5, 13, 14, 15]);
    }    
    
    #[test]
    fn test_get_neighbours_bottom_edge() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[94], vec![83, 84, 85, 93, 95]);
    }
    
    #[test]
    fn test_get_neighbours_left_edge() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[50], vec![40, 41, 51, 60, 61]);
    }    

    #[test]
    fn test_get_neighbours_right_edge() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[59], vec![48, 49, 58, 68, 69]);
    }    

    #[test]
    fn test_get_neighbours_general_case() {
        let ws = WorldState::new(10);
        assert_eq!(ws.neighbours[55], vec![44, 45, 46, 54, 56, 64, 65, 66]);        
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
    
    #[test]
    fn test_spawn_osc3() {
        let mut ws = WorldState::new(4);
        ws.spawn_osc3(0, 0);
        let init_ws = ws.clone();
        
        assert_eq!(init_ws, ws); // initially worlds are equal
        ws.next();
        assert_ne!(init_ws, ws); // iter #1 worlds are different
        ws.next();
        assert_ne!(init_ws, ws); // iter #2 worlds are different
        ws.next();
        assert_eq!(init_ws, ws); // iter #3 worlds are equal again
    }
}
