use winit::dpi::{Size, PhysicalSize};

/// The window's title.
pub const WINDOW_TITLE: &str = "Brian's Brain, by Cyprien Borée";

/// The window's width (in pixels).
pub const WINDOW_WIDTH: u32 = 800;

/// The window's height (in pixels).
pub const WINDOW_HEIGHT: u32 = 600;

/// The size of the content inside the window.
pub const WINDOW_INNER_SIZE: Size = Size::Physical(
    PhysicalSize { 
        width: WINDOW_WIDTH, 
        height: WINDOW_HEIGHT, 
    }
);
