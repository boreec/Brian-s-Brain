use std::error::Error;
use std::sync::Arc;

use vulkano::VulkanLibrary;
use vulkano::device::{Device, DeviceCreateInfo, DeviceCreationError, DeviceExtensions, 
    physical::{PhysicalDevice, PhysicalDeviceType}, Queue, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo, 
    InstanceCreationError, InstanceExtensions};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{RenderPass, RenderPassCreationError};
use vulkano::shader::{ShaderCreationError, ShaderModule};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};

use winit::window::Window;

/// Create a vulkan instance based on the installed 
/// vulkan library and required extensions for the application.
/// An error can be returned if the creation failed for any reason.
pub fn create_instance(
    library: Arc<VulkanLibrary>, 
    required_extensions: &InstanceExtensions
) -> Result <Arc<Instance>, InstanceCreationError>
 {    
    Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: *required_extensions,
            enumerate_portability: true,
            ..Default::default()        
        }
    )
}

pub fn initialize_logical_device(
    physical_device: Arc<PhysicalDevice>,
    device_extensions: &DeviceExtensions,
    queue_family_index: u32,
) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>), DeviceCreationError> {
        Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: *device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },)
}

pub fn select_device_and_queue(
    instance: Arc<Instance>,
    device_extensions: &DeviceExtensions,
    surface: Arc<Surface>
) -> Result<(Arc<PhysicalDevice>, u32), Box<dyn Error>>
{
    instance
    .enumerate_physical_devices()
    .unwrap()
    .filter(|p| {
        p.supported_extensions().contains(device_extensions)
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
   .ok_or_else(|| Box::<dyn Error>::from("No suitable device!"))
}

pub fn create_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>)
 -> Result<Arc<RenderPass>, RenderPassCreationError>
{
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
            load: Clear,
            store: Store,
            format: swapchain.image_format(),
            samples: 1,
            }
        },
        pass : {
            color: [color],
            depth_stencil: {}
        }
    )
}

pub fn create_swapchain_and_images(device: Arc<Device>, surface: Arc<Surface>)
-> Result<(Arc<Swapchain>,Vec<Arc<SwapchainImage>>), Box::<dyn Error>>
{
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();
        
    let image_format = Some(
        device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,  
    );
            
    let window = surface
        .object()
        .unwrap()
        .downcast_ref::<Window>()
        .ok_or_else(|| Box::<dyn Error>::from("failed to create window from surface!"))?;
        
    Ok(Swapchain::new(
        device,
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count,
            image_format,
            image_extent: window.inner_size().into(),
            image_usage: ImageUsage {
                color_attachment: true,
                ..Default::default()
            },
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .iter()
                .next()
                .unwrap(),
            ..Default::default()  
        },
    )?)
}
pub fn load_vertex_shader(device: Arc<Device>)
-> Result<Arc<ShaderModule>, ShaderCreationError> {
    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: 
            "#version 450

            layout(location = 0) in vec2 position;

            void main(){
                gl_Position = vec4(position, 0.0, 1.0);
            }"
        }
    }
    vs::load(device)
}

pub fn load_fragment_shader(device: Arc<Device>)
-> Result<Arc<ShaderModule>, ShaderCreationError> {
    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src:
            "#version 450

            layout(location = 0) out vec4 f_color;
            
            void main(){
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }"
        }
    }
    fs::load(device)
}

pub fn create_viewport() -> Viewport {
    Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    }
}