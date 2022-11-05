use crate::world_state::WorldState;

use clap::Parser;

use std::error::Error;

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
    
    match init_vulkan() {
        Ok(_) => {},
        Err(e) => {
            println!("Error occured while initializing Vulkan:\n {e}");
        },
    }
}

fn init_vulkan() -> Result<(), Box<dyn Error>>{
    let library = VulkanLibrary::new()?;   
    
    let instance = Instance::new(
        library,
        InstanceCreateInfo::application_from_cargo_toml()    
    )?;

    // Check if Vulkan is supported by at least one physical device.
    // If so, pick the first device to come up.
    // (to do: pick the device with the best capacities ?)
    let physical_device = instance
        .enumerate_physical_devices()?
        .next()
        .ok_or(Box::<dyn Error>::from("No physical devices support Vulkan!"));
    
    Ok(())
}
