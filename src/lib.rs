use glutin::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

pub trait RenderHandler {
    fn init(gl: &mut glow::Context) -> Self;
    fn draw(&mut self, _gl: &mut glow::Context) {}
    fn exit(&mut self, _gl: &mut glow::Context) {}
}

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

pub fn with_window<RndrHndlr: RenderHandler + 'static>() {
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Hello Window")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let windowed_context = glutin::ContextBuilder::new()
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let mut gl = unsafe {
        glow::Context::from_loader_function(|s| windowed_context.get_proc_address(s) as *const _)
    };

    let mut handler = RndrHndlr::init(&mut gl);

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                handler.draw(&mut gl);
                windowed_context.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if keycode == VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(*physical_size);
                }
                WindowEvent::CloseRequested => {
                    handler.exit(&mut gl);

                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            _ => (),
        }
    });
}
