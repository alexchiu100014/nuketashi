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

        use std::time::Instant;

        let event_loop = EventLoop::new();
        let mut buf = CpuSurface::new(&event_loop);

        let mut last_time = Instant::now();

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
                    let now = Instant::now();
                    let fps = 1.0 / (now - last_time).as_secs_f64();

                    log::debug!("fps: {:.02}", fps);

                    let target = buf.draw_begin(&()).unwrap();
                    buf.draw_end(target, &());

                    buf.request_redraw();

                    last_time = now;
                }
                _ => {}
            }
        })
    }
}
