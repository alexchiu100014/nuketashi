pub mod scene;

use crate::renderer::Renderer;
use crate::renderer::cpu::layer::LayerRenderer;

pub struct Game {
    layer: LayerRenderer,
}

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

impl Game {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        unsafe {
            use crate::platform::macos;
            macos::create_menu_bar();
        }

        let mut layer = LayerRenderer::new();
        layer.load("./test/TOHKA_02M.S25", &[1, -1, 200 + 22]);

        Game {
            layer
        }
    }

    pub fn execute(mut self) {
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

                    #[cfg(not(debug_assertions))]
                    {
                        // NOTE: we need this for profiling...
                        // should be removed in the final product.
                        println!("fps: {:.02}", fps);
                    }

                    let mut target = buf.draw_begin(&()).unwrap();
                    self.layer.render(&mut target, &());
                    buf.draw_end(target, &());

                    buf.request_redraw();

                    last_time = now;
                }
                _ => {}
            }
        })
    }
}
