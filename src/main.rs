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
        .ok_or_else(|| Box::<dyn Error>::from("No physical devices support Vulkan!"))?;
    
    // Locate a queue supporting graphical operations.
    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_, q)| q.queue_flags.graphics)
        .ok_or_else(|| Box::<dyn Error>::from("No queue family found on the device!"))?
        as u32;
    
    Ok(())
}
