use crate::world_state::WorldState;

use clap::Parser;

use std::error::Error;

use vulkano::VulkanLibrary;
use vulkano::device::DeviceCreateInfo;
use vulkano::device::Device;
use vulkano::device::QueueCreateInfo;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;

use vulkano_win::VkSurfaceBuild;

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

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
    let required_extensions = vulkano_win::required_extensions(&library);
    
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            enumerate_portability: true,
            ..Default::default()        
        },
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
 
    // Create a Vulkan context by instantiating a Device object.
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            // The queue family information
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )?;
     
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .expect("Expected to create a window for Vulkan context!");
         
    Ok(())
}
