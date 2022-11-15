use crate::WorldState;
use crate::graphics::vulkan::*;
use crate::graphics::window::*;

use std::error::Error;
use std::sync::Arc;

use vulkano::VulkanLibrary;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{allocator::StandardCommandBufferAllocator, 
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents};
use vulkano::image::{ImageAccess, SwapchainImage, view::ImageView};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{acquire_next_image, AcquireError, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo};
use vulkano::sync::{FlushError, GpuFuture, self};

use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use winit_input_helper::WinitInputHelper;

pub mod vulkan;
mod window;

pub fn run_gui(ws: &mut WorldState, framerate: u64) -> Result<(), Box<dyn Error>>{
    
    let library = VulkanLibrary::new()?;   
    let required_extensions = vulkano_win::required_extensions(&library);
    
    // 1. Create an instance of a Vulkan context.
    let instance = create_instance(&library, &required_extensions)?;

    let event_loop = EventLoop::new();

    // 2. Create a Surface, a platform-agnostic representation of the
    //    location where the image will show up (a window or a monitor).
    let surface = create_surface(&instance, &event_loop)?;
    
    let device_extensions = create_device_extensions();

    // 3. Find a physical device that can handle Vulkan's API and
    //    the required extensions for drawings.
    let (physical_device, queue_family_index) = 
        select_physical_device(&instance, &surface, &device_extensions)?;
        
    println!(
        "using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );
    
    // 4. Create a logical device, which is used as a communication channel
    //    with a physical device.
    let (device, mut queues) = 
        create_logical_device(&physical_device, &device_extensions, queue_family_index)?;
            
    // 5. Select a queue in order to submit commands buffer to the device.
    let queue = select_queue(&mut queues)?;
    
    // 6. Create a swapchain in order to render onto the Surface.
    let (mut swapchain, images) = create_swapchain_and_images(&device, &surface)?;

    // 7. Create a RenderPass object that describes the steps in
    //    which the rendering is done and subsequently the output
    //    of the graphics pipeline. 
    let render_pass = create_render_pass(&device, &swapchain)?;
    
    let vertex_buffer = create_vertex_buffer(&device, ws.as_vertices().0)?;
    
    let vs = load_vertex_shader(&device)?;
    let fs = load_fragment_shader(&device)?;
    
    
    // Create a GraphicsPipeline object to define how the
    // implementation should perform a draw operation.
    let pipeline = create_graphics_pipeline(&device, &render_pass, &vs, &fs)?;
    
    let mut viewport = create_viewport(); 
    
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
                let dimensions = get_window_dimensions(&surface);
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
                        &mut viewport
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

