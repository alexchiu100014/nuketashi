pub struct Game;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

impl Game {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        unsafe {
            use crate::platform::macos;
            macos::create_menu_bar();
        }

        Game
    }

    pub fn execute(self) {
        use crate::renderer::cpu::CpuSurface;
        use crate::renderer::{EventDelegate, RenderingSurface};

        let event_loop = EventLoop::new();
        let mut buf = CpuSurface::new(&event_loop);

        event_loop.run(move |event, _evt_loop, control_flow| {
            buf.handle_event(&event, control_flow);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::RedrawRequested(_) => {
                    let target = buf.draw_begin(&()).unwrap();
                    buf.draw_end(target, &());
                }
                _ => {}
            }
        })
    }
}
