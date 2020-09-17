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

    // Create a root context which we will render to
    let mut root_context = device.create_context(&context_descriptor, None).unwrap();

    // Define the surface type for our graphics surface ( a surface based on a native widget, i.e. not an offscreen surface )
    let surface_type = SurfaceType::Widget { native_widget };

    // Create a context for the surface that we will try to blit onto
    let mut surface_context = device
        .create_context(&context_descriptor, Some(&root_context))
        .unwrap();

    // Create a surface that can be accessed only from the GPU
    let surface = device
        .create_surface(&surface_context, SurfaceAccess::GPUOnly, surface_type)
        .unwrap();

    // Bind our surface to our surface context
    device
        .bind_surface_to_context(&mut surface_context, surface)
        .unwrap();
    // Make our root context the current context
    device.make_context_current(&root_context).unwrap();

    // Get a pointer to the OpenGL functions
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            device.get_proc_address(&surface_context, s) as *const _
        })
    };

    // Loop through render events
    let mut exit = false;
    while !exit {
        // Draw the graphics
        unsafe {
            // Create and bind a framebuffer ( this is like our swapchain framebuffer )
            let swapchain_fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(swapchain_fbo));

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

            // Render to our fbo ( again on the root context )
            // Clear the screen red on that framebuffer
            gl.clear_color(1.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            //
            // OK, blit time
            //

            // Now we need to switch to our surface context
            device.make_context_current(&surface_context).unwrap();

            // We need to create a framebuffer that we can blit from. We need to create this FBO instead of
            // just using our swapchain_fbo because that FBO was created on the root_context, and we cant
            // share FBOs across contexts.
            let surface_tmp_fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(surface_tmp_fbo));

            // Now we attach our surface FBO to the renderbuffer which *can* be shared across contexts
            gl.framebuffer_renderbuffer(
                glow::READ_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(rbo),
            );

            if !gl.check_framebuffer_status(glow::DRAW_FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE {
                panic!("Error creating framebuffer!");
            }

            // Now we bind the default framebuffer as the draw framebuffer which, in the surface_context is
            // the actual window surface
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);

            // Now we can blit from our surface_tmp_fbo and, because it is bound to the RBO that we rendered
            // to in the root context through the swapchain_fbo, we will should get an orange screen feed
            // to our window surface.
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

            gl.delete_framebuffer(surface_tmp_fbo);
            gl.delete_framebuffer(swapchain_fbo);

            let ecode = gl.get_error();
            if ecode != glow::NO_ERROR {
                panic!("GL Error! - {:#?}", Error::from_error_code(ecode));
            }
        }

        let mut surface = device
            .unbind_surface_from_context(&mut surface_context)
            .unwrap()
            .unwrap();
        device
            .present_surface(&surface_context, &mut surface)
            .unwrap();
        device
            .bind_surface_to_context(&mut surface_context, surface)
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

    device.destroy_context(&mut surface_context).unwrap();
    device.destroy_context(&mut root_context).unwrap();
}
