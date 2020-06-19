pub mod scene;

use crate::renderer::cpu::layer::LayerRenderer;
use crate::renderer::Renderer;

use crate::script::mil::command::Command as MilCommand;

pub struct Game {
    layers: Vec<LayerRenderer>,
    commands: Vec<MilCommand>,
    waiting: bool,
}

use winit::event::{Event, WindowEvent, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};

impl Game {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        unsafe {
            use crate::platform::macos;
            macos::create_menu_bar();
        }

        let mut layers = vec![];
        layers.resize_with(30, || LayerRenderer::new());

        Game {
            layers,
            commands: vec![],
            waiting: false,
        }
    }

    pub fn load_script(&mut self) {
        use encoding_rs::SHIFT_JIS;

        use crate::script::rio::parser::Parser;
        use crate::script::rio::transpiler::Transpiler;

        let script = include_bytes!("../../testcase/02_NK_23H copy.TXT");
        let (script, _, _) = SHIFT_JIS.decode(script);

        let mut parser = Parser::from_raw_bytes(script.as_bytes());
        let script = parser.parse().unwrap();

        let tr = Transpiler::new(script);
        let script = tr.transpile();

        use crate::script::mil::pass::Pass;
        use crate::script::mil::pass::prefetch::PrefetchPass;

        let prefetch = PrefetchPass::new();
        let mut script = prefetch.process(script);

        script.reverse();

        self.commands = script;
    }

    pub fn exec_script(&mut self) {
        if self.waiting {
            return;
        }

        use crate::script::mil::command::*;

        while let Some(cmd) = self.commands.pop() {
            match cmd {
                MilCommand::RuntimeCommand(RuntimeCommand::WaitUntilUserEvent) => {
                    self.waiting = true;
                    return;
                }
                MilCommand::LayerCommand {
                    layer_no, command
                } => {
                    self.layers[layer_no as usize].send(command);
                }
                _ => {
                    log::debug!("skipped command: {:?}", cmd);
                }
            }
        }
    }

    pub fn execute(mut self) {
        use crate::renderer::cpu::CpuSurface;
        use crate::renderer::{EventDelegate, RenderingSurface};

        self.load_script();

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
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { 
                        state: ElementState::Released,
                        .. },
                    ..
                } => {
                    self.waiting = false;
                }
                Event::RedrawRequested(_) => {
                    let now = Instant::now();
                    let fps = 1.0 / (now - last_time).as_secs_f64();

                    log::debug!("fps: {:.02}", fps);

                    self.exec_script();

                    #[cfg(not(debug_assertions))]
                    {
                        // NOTE: we need this for profiling...
                        // should be removed in the final product.
                        println!("fps: {:.02}", fps);
                    }

                    let mut target = buf.draw_begin(&()).unwrap();

                    use rayon::prelude::*;

                    self.layers.par_iter_mut().for_each(|l| l.update());

                    for l in &mut self.layers {
                        l.render(&mut target, &());
                    }

                    buf.draw_end(target, &());

                    buf.request_redraw();

                    last_time = now;
                }
                _ => {}
            }
        })
    }
}
