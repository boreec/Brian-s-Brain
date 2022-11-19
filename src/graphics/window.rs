use std::cmp::Ordering;
use std::error::Error;
use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::swapchain::Surface;

use vulkano_win::VkSurfaceBuild;

use winit::dpi::{Size, PhysicalPosition, PhysicalSize, Position};
use winit::event_loop::EventLoop;
use winit::monitor::MonitorHandle;
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

/// Create the surface, which is the graphical link between the framebuffers
/// and the actual window on the desktop environment.
pub fn create_surface(instance: &Arc<Instance>, event_loop: &EventLoop<()>)
-> Result<Arc<Surface>, Box<dyn Error>> 
{
    
    let monitor = select_biggest_monitor(event_loop)
        .ok_or_else(|| Box::<dyn Error>::from("No monitors found for GUI."))?;
    
    let monitor_size = monitor.size();
    let centered_h = monitor_size.height / 2 - WINDOW_HEIGHT / 2;
    let centered_w = monitor_size.width / 2 - WINDOW_WIDTH / 2;

    let centered_pos : Position = Position::Physical(
        PhysicalPosition::new(centered_w as i32, centered_h as i32) 
    );
    
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_min_inner_size(WINDOW_INNER_SIZE)
        .with_max_inner_size(WINDOW_INNER_SIZE)
        .with_title(String::from(WINDOW_TITLE))
        .with_position(centered_pos)
        .build_vk_surface(event_loop, instance.clone())?;
    
    Ok(window)
}

/// Select the biggest monitor to display the window.
fn select_biggest_monitor(event_loop: &EventLoop<()>) -> Option<MonitorHandle> {
    let mut monitors = event_loop.available_monitors();
    let mut best_monitor = monitors.next();
    while monitors.size_hint().0 > 0 {
        let tmp = monitors.next();
        if tmp.cmp(&best_monitor) == Ordering::Greater {
            best_monitor = tmp;
        }
    }
    best_monitor
}

/// Return the dimensions of the windows (as `PhisicalSize` pixels).
pub fn get_window_dimensions(surface: &Surface) -> PhysicalSize<u32> {
    let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
    window.inner_size()
}