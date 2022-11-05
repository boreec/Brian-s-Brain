use crate::world_state::WorldState;

use clap::Parser;

use vulkano::VulkanLibrary;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;

mod world_state;

/// Program to run the Brian's Brain cellular automaton.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    
    /// The size of the world in which the cells live.    
    #[arg(short, long, default_value_t = 10)]
    size: u16,     
}

fn main() {
    let args = Args::parse();
    
    let ws = WorldState::new(args.size);
    
    init_vulkan();
}

fn init_vulkan() {
    let library = VulkanLibrary::new().expect("no Vulkan library/DLL");   
}
