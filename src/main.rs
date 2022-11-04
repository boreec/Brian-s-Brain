use crate::world_state::WorldState;

use clap::Parser;

mod world_state;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    
    #[arg(short, long, default_value_t = 10)]
    size: u16,     
}

fn main() {
    let args = Args::parse();
    
    let ws = WorldState::new(args.size);
}
