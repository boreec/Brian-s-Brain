use bytemuck::Pod;
use bytemuck::Zeroable;

use std::error::Error;
use std::sync::Arc;

use vulkano::VulkanLibrary;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::command_buffer::SubpassContents;
use vulkano::device::Device;
use vulkano::device::DeviceCreateInfo;
use vulkano::device::DeviceCreationError;
use vulkano::device::DeviceExtensions;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::Queue;
use vulkano::device::QueueCreateInfo;
use vulkano::image::ImageAccess;
use vulkano::image::ImageUsage;
use vulkano::image::SwapchainImage;
use vulkano::image::view::ImageView;
use vulkano::impl_vertex;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::render_pass::RenderPass;
use vulkano::render_pass::Subpass;
use vulkano::sync;
use vulkano::sync::FlushError;
use vulkano::sync::GpuFuture;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::acquire_next_image;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::swapchain::SwapchainPresentInfo;

use vulkano_win::VkSurfaceBuild;

use winit::event::Event;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

use winit_input_helper::WinitInputHelper;

pub fn init_vulkan() -> Result<(), Box<dyn Error>>{
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
        .unwrap();
        
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
    
    // Create logical device
    let (device, mut queues) = 
        initialize_logical_device(
            physical_device, 
            &device_extensions, 
            queue_family_index
        )?; // failed to create logical device
            
    let queue = queues
        .next()
        .ok_or(Box::<dyn Error>::from("failed to retrieve queue!"))?;
    
    let (mut swapchain, images) = {
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
            
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
        
        Swapchain::new(
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
        )
        .unwrap()
    };
    
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
    
    // use repr(C) to prevent rust to mess with the data.
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
    struct Vertex {
        position: [f32; 2],
    }
    impl_vertex!(Vertex, position);

    // Vertices representing a triangle.
    let vertices = [
        Vertex {
            position: [-0.5, -0.25],
        },
        Vertex {
            position: [0.0, 0.5],
        },
        Vertex {
            position: [0.25, -0.1],
        },
    ];
    
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            vertex_buffer: true,
            ..Default::default()
        },
        false,
        vertices
    )
    .unwrap();
    
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
    
    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();
    
    // Build a RenderPass object to represent the steps in which
    // the rendering is done. It contains three parts: 
    // 1 - List of attachments (image views)
    // 2 - Subpasses
    // 3 - Dependencies
    let render_pass = vulkano::single_pass_renderpass!(
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
    .unwrap();
    
    // Create a GraphicsPipeline object to define how the
    // implementation should perform a draw operation.
    let pipeline = GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .build(device.clone())
        .unwrap();
    
    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);    
    
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());
    
    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());
    
    let mut input = WinitInputHelper::new();
    event_loop.run(move |event, _, control_flow| {
        if input.update(&event){
            if input.key_released(VirtualKeyCode::Escape) {
                *control_flow = ControlFlow::Exit;
            }
        }
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }
            Event::RedrawEventsCleared => {
                let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
                let dimensions = window.inner_size();
                // Don't draw frame if one dimension is equal to 0.
                if dimensions.width == 0 || dimensions.height == 0 {
                    return;
                }
                
                previous_frame_end.as_mut().unwrap().cleanup_finished();
                
                if recreate_swapchain {
                    let (new_swapchain, new_images) =
                        match swapchain.recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..swapchain.create_info()
                    }) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::ImageExtentNotSupported {..}) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };
                    
                    swapchain = new_swapchain;
                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                    );
                    recreate_swapchain = false;
                }
                
                // Try to acquire image from Swapchain
                let (image_index, suboptimal, acquire_future) = 
                    match acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };
            
                if suboptimal {
                    recreate_swapchain = true;
                }
                
                let mut builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();
                
                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0,0.0,1.0,1.0].into())],
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
                
                let command_buffer = builder.build().unwrap();
                
                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();
                
                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        panic!("Failed to flush future: {:?}", e);
                    }
                }
            }
            _ => {}
        }
    });
    Ok(())
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
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

fn initialize_logical_device(
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
