pub mod scene;

use crate::renderer::vulkano::layer::LayerRenderer;
use crate::renderer::Renderer;
use crate::script::mil::command::{Command as MilCommand, RendererCommand, RuntimeCommand};

use vulkano::framebuffer::RenderPassAbstract;

use std::sync::Arc;

pub struct Game {
    layers: Vec<LayerRenderer>,
    // face_layer: LayerRenderer,
    // text_layer: Image,
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
                /* use crate::renderer::common::text;
                const FONT_HEIGHT: f32 = 44.0;

                self.text_layer.clear();

                text::write_text_in_box(
                    text::create_font(),
                    FONT_HEIGHT,
                    &format!("{}\n{}", name.unwrap_or_default(), dialogue),
                    (self.text_layer.width, self.text_layer.height),
                    &mut self.text_layer.rgba_buffer,
                ); */
            }
            _ => {
                log::debug!("skipped renderer command: {:?}", command);
            }
        }
    }

    pub fn execute(mut self) {
        use crate::renderer::vulkano::surface::VulkanoSurface;
        use crate::renderer::{EventDelegate, RenderingSurface};

        self.load_script();

        use std::time::Instant;

        let event_loop = EventLoop::new();
        let mut buf = VulkanoSurface::new(&event_loop);

        let mut last_time = Instant::now();

        use crate::renderer::vulkano::layer::LayerRenderingContext;
        use crate::renderer::vulkano::pipeline;

        let render_pass = pipeline::create_render_pass(buf.device.clone(), buf.format())
            as Arc<dyn RenderPassAbstract + Sync + Send>;
        let pipeline = pipeline::create_pict_layer_pipeline(buf.device.clone(), render_pass.clone());

        let ctx = LayerRenderingContext {
            render_pass,
            pipeline: pipeline.clone(),
        };

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

                    let mut target = buf.draw_begin(&ctx).unwrap();

                    target
                        .command_buffer
                        .begin_render_pass(
                            target.framebuffer.clone(),
                            false,
                            vec![[0.0, 0.0, 0.0, 1.0].into()],
                        )
                        .unwrap();

                    for l in &mut self.layers {
                        l.update(buf.graphical_queue.clone(), pipeline.clone());
                        l.take_future(buf.device.clone()).flush().unwrap();
                        l.render(&mut target, &ctx);
                    }

                    target.command_buffer.end_render_pass().unwrap();

                    buf.draw_end(target, &ctx);
                    buf.surface.window().request_redraw();

                    last_time = now;
                }
                _ => {}
            }
        })
    }
}
