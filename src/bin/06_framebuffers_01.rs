use glow::HasContext;
use surfman::{
    Connection, ContextAttributeFlags, ContextAttributes, GLVersion, SurfaceAccess, SurfaceType,
};
use winit::{
    dpi::PhysicalSize, DeviceEvent, Event, EventsLoop, KeyboardInput, VirtualKeyCode,
    WindowBuilder, WindowEvent,
};

surfman::declare_surfman!();

pub trait SliceAsBytes<T> {
    fn as_mem_bytes(&self) -> &[u8];
}

impl<T: AsRef<[U]>, U> SliceAsBytes<U> for T {
    fn as_mem_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().as_ptr() as *const u8,
                std::mem::size_of::<T>() * self.as_ref().len(),
            )
        }
    }
}

// From GFX:
// https://github.com/katharostech/gfx/blob/77c3e28331f8ab593e57425b47db344f0e9e8112/src/backend/gl/src/lib.rs#L162
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Error {
    NoError,
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    InvalidFramebufferOperation,
    OutOfMemory,
    UnknownError,
}

impl Error {
    pub fn from_error_code(error_code: u32) -> Error {
        match error_code {
            glow::NO_ERROR => Error::NoError,
            glow::INVALID_ENUM => Error::InvalidEnum,
            glow::INVALID_VALUE => Error::InvalidValue,
            glow::INVALID_OPERATION => Error::InvalidOperation,
            glow::INVALID_FRAMEBUFFER_OPERATION => Error::InvalidFramebufferOperation,
            glow::OUT_OF_MEMORY => Error::OutOfMemory,
            _ => Error::UnknownError,
        }
    }
}

pub fn main() {
    // Create the window event loop
    let mut event_loop = EventsLoop::new();
    // Obtain the screen scaling factor
    let scale_factor = event_loop.get_primary_monitor().get_hidpi_factor();
    // Create a new logical size for the window based on the desired physical size
    let logical_size = PhysicalSize::new(800f64, 600f64).to_logical(scale_factor);
    // Create a window
    let window = WindowBuilder::new()
        .with_title("Me Learning OpenGL")
        .with_dimensions(logical_size)
        .build(&event_loop)
        .unwrap();

    // Show the window
    window.show();

    // Create a connection to the graphics provider from our winit window
    let conn = Connection::from_winit_window(&window).unwrap();
    // Create a native widget to attach the visible render surface to
    let native_widget = conn
        .create_native_widget_from_winit_window(&window)
        .unwrap();
    // Create a hardware adapter that we can used to create graphics devices from
    let adapter = conn.create_hardware_adapter().unwrap();
    // Create a graphics device using our hardware adapter
    let mut device = conn.create_device(&adapter).unwrap();

    // Define the attributes for our OpenGL context
    let context_attributes = ContextAttributes {
        version: GLVersion::new(3, 3),
        flags: ContextAttributeFlags::ALPHA
            | ContextAttributeFlags::DEPTH
            | ContextAttributeFlags::STENCIL,
    };

    // Create a context descriptor based on our defined context attributes
    let context_descriptor = device
        .create_context_descriptor(&context_attributes)
        .unwrap();
    // Define the surface type for our graphics surface ( a surface based on a native widget, i.e. not an offscreen surface )
    let surface_type = SurfaceType::Widget { native_widget };
    // Create an OpenGL context
    let mut context = device.create_context(&context_descriptor, None).unwrap();

    // Create a surface that can be accessed only from the GPU
    let surface = device
        .create_surface(&context, SurfaceAccess::GPUOnly, surface_type)
        .unwrap();

    // Bind our surface to our create GL context
    device
        .bind_surface_to_context(&mut context, surface)
        .unwrap();
    // Make our context the current context
    device.make_context_current(&context).unwrap();

    // Get a pointer to the OpenGL functions
    let mut gl = unsafe {
        glow::Context::from_loader_function(|s| device.get_proc_address(&context, s) as *const _)
    };

    // Loop through render events
    let mut exit = false;
    while !exit {
        // Draw the graphics
        unsafe {
            // Create and bind framebuffer
            let fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(fbo));
            // Create and bind renderbuffer
            let rbo = gl.create_renderbuffer().unwrap();
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::RGB, 800, 600);

            // Attach renderbuffer to framebuffer
            gl.framebuffer_renderbuffer(
                glow::DRAW_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(rbo),
            );
            if !gl.check_framebuffer_status(glow::DRAW_FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE {
                panic!("Error creating framebuffer!");
            }

            // Clear the screen red on that framebuffer
            gl.clear_color(1.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            // Bind framebuffer 0 as our draw buffer
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);

            let fbo2 = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fbo2));
            gl.framebuffer_renderbuffer(
                glow::READ_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(rbo),
            );

            gl.blit_framebuffer(
                0,
                0,
                800,
                600,
                0,
                0,
                800,
                600,
                glow::COLOR_BUFFER_BIT,
                glow::LINEAR,
            );

            let ecode = gl.get_error();
            if ecode != glow::NO_ERROR {
                panic!("GL Error! - {:#?}", Error::from_error_code(ecode));
            }
        }

        let mut surface = device
            .unbind_surface_from_context(&mut context)
            .unwrap()
            .unwrap();
        device.present_surface(&context, &mut surface).unwrap();
        device
            .bind_surface_to_context(&mut context, surface)
            .unwrap();

        // Handle events
        event_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::Destroyed,
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::DeviceEvent {
                event:
                    DeviceEvent::Key(KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    }),
                ..
            } => exit = true,
            _ => {}
        });
    }

    device.destroy_context(&mut context).unwrap();
}
