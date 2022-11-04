pub struct WorldState {
    size: u16,
}

impl WorldState {
    pub fn new(w_size: u16) -> WorldState {
        WorldState {
            size: w_size,
        }
    }
}
