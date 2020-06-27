pub mod scene;

use crate::renderer::cpu::layer::LayerRenderer;
use crate::renderer::Renderer;

use crate::renderer::cpu::image::Image;

use crate::script::mil::command::{Command as MilCommand, RendererCommand, RuntimeCommand};

pub struct Game {
    layers: Vec<LayerRenderer>,
    face_layer: LayerRenderer,
    text_layer: Image,
    commands: Vec<MilCommand>,
    waiting: bool,
}

use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

impl Game {
    fn platform_specific_setup() {
        #[cfg(target_os = "macos")]
        unsafe {
            use crate::platform::macos;
            macos::create_menu_bar();
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self::platform_specific_setup();

        let mut layers = vec![];
        layers.resize_with(30, || LayerRenderer::new());

        Game {
            layers,
            commands: vec![],
            waiting: false,
            face_layer: LayerRenderer::new(),
            text_layer: Image::new(900, 300),
        }
    }

    pub fn load_script(&mut self) {
        use encoding_rs::SHIFT_JIS;

        use crate::script::rio::parser::Parser;
        use crate::script::rio::transpiler::Transpiler;

        let script = std::fs::read("./testcase/02_NK_23H.TXT").unwrap();
        let (script, _, _) = SHIFT_JIS.decode(&script);

        let mut parser = Parser::from_raw_bytes(script.as_bytes());
        let script = parser.parse().unwrap();

        let tr = Transpiler::new(script);
        let script = tr.transpile();

        use crate::script::mil::pass::prefetch::PrefetchPass;
        use crate::script::mil::pass::Pass;

        let prefetch = PrefetchPass::new();
        let mut script = prefetch.process(script);

        script.reverse();

        self.commands = script;
    }

    pub fn exec_script(&mut self) {
        if self.waiting {
            return;
        }

        while let Some(cmd) = self.commands.pop() {
            match cmd {
                MilCommand::RuntimeCommand(RuntimeCommand::WaitUntilUserEvent) => {
                    self.waiting = true;
                    return;
                }
                MilCommand::RendererCommand(r) => {
                    self.visit_renderer_command(r);
                }
                MilCommand::LayerCommand { layer_no, command } => {
                    self.layers[layer_no as usize].send(command);
                }
                _ => {
                    log::debug!("skipped command: {:?}", cmd);
                }
            }
        }
    }

    fn visit_renderer_command(&mut self, command: RendererCommand) {
        match command {
            RendererCommand::Dialogue(name, dialogue) => {
                use crate::renderer::common::text;
                const FONT_HEIGHT: f32 = 44.0;

                self.text_layer.clear();

                text::write_text_in_box(
                    text::create_font(),
                    FONT_HEIGHT,
                    &format!("{}\n{}", name.unwrap_or_default(), dialogue),
                    (self.text_layer.width, self.text_layer.height),
                    &mut self.text_layer.rgba_buffer,
                );
            }
            _ => {
                log::debug!("skipped renderer command: {:?}", command);
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
                    event:
                        WindowEvent::MouseInput {
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    self.waiting = false;
                }
                Event::RedrawRequested(_) => {
                    let now = Instant::now();
                    let fps = 1.0 / (now - last_time).as_secs_f64();

                    log::debug!("fps: {:.02}", fps);

                    self.exec_script();

                    let mut target = buf.draw_begin(&()).unwrap();

                    use rayon::prelude::*;

                    self.layers.par_iter_mut().for_each(|l| l.update());

                    for l in &mut self.layers {
                        l.render(&mut target, &());
                    }

                    // draw text & face layer
                    for i in 0..=2 {
                        for j in 0..=2 {
                            target.draw_image_colored(
                                &self.text_layer.rgba_buffer,
                                (378 + (i << 1), 638 + (j << 1)),
                                (900, 300),
                                1.0,
                                [0, 0, 0],
                            );
                        }
                    }

                    target.draw_image_colored(
                        &self.text_layer.rgba_buffer,
                        (380, 640),
                        (900, 300),
                        1.0,
                        [255, 255, 255],
                    );

                    self.face_layer.render(&mut target, &());

                    buf.draw_end(target, &());

                    buf.request_redraw();

                    last_time = now;
                }
                _ => {}
            }
        })
    }
}
