use crate::WorldState;
use crate::graphics::vulkan::*;
use crate::graphics::window::*;

use std::error::Error;

use vulkano::VulkanLibrary;
use vulkano::swapchain::{acquire_next_image, AcquireError, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo};
use vulkano::sync::{FlushError, GpuFuture, self};

use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use winit_input_helper::WinitInputHelper;

pub mod vulkan;
mod window;

pub fn run_gui(mut ws: WorldState, framerate: u64) -> Result<(), Box<dyn Error>>{
    
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
    
    let mut viewport = create_viewport();
    
    // 8. Create the actual buffers to be able to display images.
    let mut framebuffers = get_framebuffers(&images, &render_pass, &mut viewport);
    
    // 9. Create the vertex buffer
    let mut vertex_buffer = create_vertex_buffer(&device, ws.as_vertices().0)?;
    
    // 10. Load the shaders.
    let vs = load_vertex_shader(&device)?;
    let fs = load_fragment_shader(&device)?;
    
    // 11. Create the graphics pipeline.
    let pipeline = create_graphics_pipeline(&device, &render_pass, &vs, &fs)?;    
    
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
                    framebuffers = get_framebuffers(&new_images, &render_pass, &mut viewport);
                    recreate_swapchain = false;
                }
                
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
                
                let command_buffer = get_command_buffer(
                    &device, 
                    &queue,
                    &pipeline,
                    &vertex_buffer,
                    &viewport,
                    &framebuffers, 
                    image_index
                );
                
                let command_buffer = match command_buffer {
                    Ok(r) => r,
                    Err(e) => {panic!("Failed to create command buffer: {:?}", e);}
                };

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
                        ws.next();
                        vertex_buffer = create_vertex_buffer(&device, ws.as_vertices().0).unwrap();
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
}

