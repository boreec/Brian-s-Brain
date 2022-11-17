use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::swapchain::Surface;

use vulkano_win::CreationError;
use vulkano_win::VkSurfaceBuild;

use winit::dpi::{Size, PhysicalSize};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

/// The window's title.
pub const WINDOW_TITLE: &str = "Brian's Brain, by Cyprien Borée";

/// The window's width (in pixels).
pub const WINDOW_WIDTH: u32 = 1000;

/// The window's height (in pixels).
pub const WINDOW_HEIGHT: u32 = 1000;

/// The size of the content inside the window.
pub const WINDOW_INNER_SIZE: Size = Size::Physical(
    PhysicalSize { 
        width: WINDOW_WIDTH, 
        height: WINDOW_HEIGHT, 
    }
);

pub fn create_surface(instance: &Arc<Instance>, event_loop: &EventLoop<()>)
-> Result<Arc<Surface>, CreationError> 
{
    WindowBuilder::new()
        .with_resizable(false)
        .with_min_inner_size(WINDOW_INNER_SIZE)
        .with_max_inner_size(WINDOW_INNER_SIZE)
        .with_title(String::from(WINDOW_TITLE))
        .build_vk_surface(event_loop, instance.clone())
}

pub fn get_window_dimensions(surface: &Surface) -> PhysicalSize<u32> {
    let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
    window.inner_size()
}