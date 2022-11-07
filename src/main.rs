use crate::world_state::WorldState;

use clap::Parser;

use std::error::Error;

use vulkano::VulkanLibrary;
use vulkano::device::Device;
use vulkano::device::DeviceCreateInfo;
use vulkano::device::DeviceExtensions;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::QueueCreateInfo;
use vulkano::device::QueueFlags;
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

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .expect("Expected to create a window for Vulkan context!");
    
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()    
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| {
            p.supported_extensions().contains(&device_extensions)
        })
        // for a device supporting vulkan check if it contains
        // queues that support graphical operations.
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)|{
                    q.queue_flags.graphics == true
                        && p.surface_support(i as u32, &surface)
                            .unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        // Set a priority for each physical device according to its type.
        .min_by_key(|(p, _)| {
            match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            }
        })
        .expect("No suitable physical device found");
    
    println!(
        "using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );
    
    // Initializing the logical device
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )?;
    
    Ok(())
}
