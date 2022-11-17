use bytemuck::{Pod, Zeroable};

use std::error::Error;
use std::sync::Arc;

use vulkano::VulkanLibrary;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{allocator::StandardCommandBufferAllocator,
    AutoCommandBufferBuilder, BuildError, CommandBufferUsage, 
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents};
use vulkano::device::{Device, DeviceCreateInfo, DeviceCreationError, DeviceExtensions, 
    physical::{PhysicalDevice, PhysicalDeviceType}, Queue, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo, 
    InstanceCreationError, InstanceExtensions};
use vulkano::image::{ImageAccess, ImageUsage, SwapchainImage};
use vulkano::image::view::ImageView;
use vulkano::impl_vertex;
use vulkano::memory::allocator::{AllocationCreationError, StandardMemoryAllocator};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::GraphicsPipelineCreationError;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, 
    RenderPassCreationError, Subpass};
use vulkano::shader::{ShaderCreationError, ShaderModule};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};

use winit::window::Window;

// use repr(C) to prevent rust to mess with the data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}
impl_vertex!(Vertex, position);
/// vulkan library and required extensions for the application.
/// An error can be returned if the creation failed for any reason.
pub fn create_instance(
    library: &Arc<VulkanLibrary>, 
    required_extensions: &InstanceExtensions
) -> Result <Arc<Instance>, InstanceCreationError>
 {    
    Instance::new(
        library.clone(),
        InstanceCreateInfo {
            enabled_extensions: *required_extensions,
            enumerate_portability: true,
            ..Default::default()        
        }
    )
}

pub fn create_logical_device(
    physical_device: &Arc<PhysicalDevice>,
    device_extensions: &DeviceExtensions,
    queue_family_index: u32,
) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>), DeviceCreationError> {
        Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            enabled_extensions: *device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },)
}

/// Select the best physical device suited for the designed tasks.
/// In other words, find if there's a device supporting the given
/// extensions. If more than one device is found, the *best* one is
/// returned, otherwise if no device has been found, an error is propagated.
pub fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions
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
                q.queue_flags.graphics && p.surface_support(i as u32, surface).unwrap_or(false)
            })
            .map(|i| (p, i as u32))
    }) // set a priority for each device according to its type
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

pub fn select_queue(queues: &mut(impl ExactSizeIterator + Iterator<Item = Arc<Queue>>))
-> Result<Arc<Queue>, Box<dyn Error>> {
    queues
        .next()
        .ok_or_else(|| Box::<dyn Error>::from("failed to retrieve queue!"))
}

pub fn create_render_pass(device: &Arc<Device>, swapchain: &Arc<Swapchain>)
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

pub fn create_swapchain_and_images(device: &Arc<Device>, surface: &Arc<Surface>)
-> Result<(Arc<Swapchain>,Vec<Arc<SwapchainImage>>), Box::<dyn Error>>
{
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(surface, Default::default())
        .unwrap();
        
    let image_format = Some(
        device
            .physical_device()
            .surface_formats(surface, Default::default())
            .unwrap()[0]
            .0,  
    );
            
    let window = surface
        .object()
        .unwrap()
        .downcast_ref::<Window>()
        .ok_or_else(|| Box::<dyn Error>::from("failed to create window from surface!"))?;
        
    Ok(Swapchain::new(
        device.clone(),
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

pub fn create_vertex_buffer(device: &Arc<Device>, vertices: Vec<Vertex>)
-> Result<Arc<CpuAccessibleBuffer<[Vertex]>>, AllocationCreationError>
{
    CpuAccessibleBuffer::from_iter(
        &StandardMemoryAllocator::new_default(device.clone()),
        BufferUsage {
            vertex_buffer: true,
            ..Default::default()
        },
        false,
        vertices
    )
}

pub fn load_vertex_shader(device: &Arc<Device>)
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
    vs::load(device.clone())
}

pub fn load_fragment_shader(device: &Arc<Device>)
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
    fs::load(device.clone())
}

pub fn create_device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    }
}

pub fn create_viewport() -> Viewport {
    Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    }
}

pub fn create_graphics_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
    vs: &Arc<ShaderModule>,
    fs: &Arc<ShaderModule>
) -> Result<Arc<GraphicsPipeline>, GraphicsPipelineCreationError> {
        GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .build(device.clone())
}

pub fn get_framebuffers(
    images: &[Arc<SwapchainImage>],
    render_pass: &Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
    
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
    .collect::<Vec<_>>()
}

pub fn get_command_buffer(
    device: &Arc<Device>,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    vertex_buffer: &Arc<CpuAccessibleBuffer<[Vertex]>>,
    viewport: &Viewport,
    framebuffers: &[Arc<Framebuffer>],
    image_index: u32
)
-> Result<PrimaryAutoCommandBuffer, BuildError>
{
    // Try to acquire image from Swapchain
    
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let mut builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some([0.,0.,0.,1.].into())],
                ..RenderPassBeginInfo::framebuffer(
                    framebuffers[image_index as usize].clone(),
                )
            },
            SubpassContents::Inline,
        )
        .unwrap()
        .set_viewport(0, [viewport.clone()])
        .bind_pipeline_graphics(pipeline.clone())
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .draw(vertex_buffer.len() as u32, 1, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap();

    builder.build()
}