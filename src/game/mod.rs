// TODO: refactor

pub mod instance;
pub mod layer;
pub mod pipeline;
pub mod shaders;
pub mod text;
pub mod texture_loader;

// vulkano; Vulkan rapper
use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::swapchain::SwapchainImage;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{FullscreenExclusive, PresentMode, Surface, SurfaceTransform, Swapchain};

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;

use crate::constants;
use crate::script::vm::{DrawCall, Vm};

pub struct Game<'a> {
    pub physical: PhysicalDevice<'a>,
    pub device: Arc<Device>,
    pub event_loop: Option<EventLoop<()>>,
    pub surface: Arc<Surface<Window>>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub images: Vec<Arc<SwapchainImage<Window>>>,
    pub graphical_queue: Arc<Queue>,
    pub transfer_queue: Arc<Queue>,
    pub vm: Vm<std::io::Cursor<String>>,
}

impl Game<'static> {
    pub fn new() -> Self {
        /*
         * Vulkan-based program should follow these instructions to ininitalize:
         *
         * - Create an instance
         * - Obtain a physical device
         * - Create a Vulkan surface from Window
         *   - This requires the creation of a winit Window.
         * - Create a device
         */

        let event_loop = Some(EventLoop::new());

        #[cfg(target_os = "macos")]
        unsafe {
            // Create a menu-bar for macOS.
            crate::platform::macos::create_menu_bar();
        }

        let physical = Self::create_physical();
        let surface = Self::create_window(event_loop.as_ref().unwrap());
        let (device, graphical_queue, transfer_queue) = Self::create_device(physical, &surface);

        let caps = surface.capabilities(physical).unwrap();
        use vulkano::format::Format;

        log::debug!("supported formats: {:?}", caps.supported_formats);

        let (f, cs) = caps
            .supported_formats
            .iter()
            .copied()
            .find(|(f, _)| {
                *f == Format::B8G8R8A8Srgb
                    || *f == Format::B8G8R8Srgb
                    || *f == Format::R8G8B8A8Srgb
                    || *f == Format::R8G8B8Srgb
            })
            .unwrap();

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            f,
            surface.window().inner_size().into(),
            1,
            caps.supported_usage_flags,
            &graphical_queue,
            SurfaceTransform::Identity,
            caps.supported_composite_alpha.iter().next().unwrap(),
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            cs,
        )
        .expect("failed to create a swapchain");

        // Create VM.
        let mut vm = Vm::new(std::io::Cursor::new({
            use encoding_rs::SHIFT_JIS;
            let script = include_bytes!("../../blob/NUKITASHI_T.WAR/02_NK_03.TXT");
            let (v, _, _) = SHIFT_JIS.decode(script);
            v.into()
        }));

        vm.construct_face_map("./blob/NUKITASHI_T.WAR/FAUTOTBL.BIN")
            .expect("failed to construct facetable");

        Game {
            physical,
            device,
            event_loop,
            surface,
            swapchain,
            images,
            graphical_queue,
            transfer_queue,
            vm,
        }
    }
}

// -- Initialization
impl<'a> Game<'a> {
    fn create_physical() -> PhysicalDevice<'static> {
        let instance = instance::get_instance();

        // Obtain a physical device.
        // Note that a PhysicalDevice is bound to the reference of the instance,
        // hence the instance should be alive while `physical` is alive.
        // Instance has 'static lifetime parameter, so no problem here.
        //
        // TODO: let users to choose physical devices
        let physical = PhysicalDevice::enumerate(instance)
            .next()
            .expect("no physical device available");

        log::debug!("device: {}, type: {:?}", physical.name(), physical.ty());

        physical
    }

    fn create_window(event_loop: &EventLoop<()>) -> Arc<Surface<Window>> {
        use winit::dpi::LogicalSize;

        let window = WindowBuilder::new()
            .with_title(constants::GAME_ENGINE_FULL_NAME)
            .with_inner_size(LogicalSize {
                width: constants::GAME_WINDOW_WIDTH,
                height: constants::GAME_WINDOW_HEIGHT,
            })
            // .with_resizable(false)
            .build(event_loop)
            .expect("failed to build Window");

        log::debug!(
            "created Window; size: {}, {}",
            constants::GAME_WINDOW_WIDTH,
            constants::GAME_WINDOW_HEIGHT
        );

        let surface = vulkano_win::create_vk_surface(window, instance::get_instance().clone())
            .expect("failed to build Vulkan surface");

        log::debug!("created Vulkan surface");

        surface
    }

    fn create_device<T>(
        physical: PhysicalDevice,
        surface: &Surface<T>,
    ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        let gr_queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false));

        let tr_queue_family = physical.queue_families().find(|&q| {
            (q.supports_graphics() || q.supports_compute()) // VK_QUEUE_TRANSFER_BIT
                && gr_queue_family != Some(q) // no overlap
        });

        let extensions = DeviceExtensions {
            khr_swapchain: true, // swapchain is required
            ..DeviceExtensions::none()
        };

        let (d, mut q) = Device::new(
            physical,
            physical.supported_features(),
            &extensions,
            vec![(gr_queue_family, 1.0), (tr_queue_family, 0.5)]
                .into_iter()
                .filter_map(|(v, a)| Some((v?, a))),
        )
        .expect("failed to create device");

        // graphics queue and transfer queue
        let gq = q.next().unwrap();
        let tq = q.next().unwrap_or_else(|| gq.clone());

        log::debug!("created device and queue");

        (d, gq, tq)
    }
}

// -- Run-loop execution & event-handling
impl Game<'static> {
    /// Executes an event loop.
    ///
    /// It takes the ownership of a Game instance, and won't return until
    /// the program is closed.
    pub fn execute(mut self) {
        use vulkano::command_buffer::AutoCommandBufferBuilder;
        use vulkano::swapchain::{AcquireError, SwapchainCreationError};
        use vulkano::sync::{FlushError, GpuFuture};

        use crate::game::text::Text;

        use crate::game::layer::Layer;

        use std::time::Instant;

        let render_pass = pipeline::create_render_pass(self.device.clone(), &self.swapchain);

        let pipeline =
            pipeline::create_pict_layer_pipeline(self.device.clone(), render_pass.clone());

        let pipeline_text =
            pipeline::create_text_layer_pipeline(self.device.clone(), render_pass.clone());

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        let mut framebuffers =
            window_size_dependent_setup(&self.images, render_pass.clone(), &mut dynamic_state);
        let mut previous_frame_end =
            Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<dyn GpuFuture>);
        let mut recreate_swapchain = false;

        let mut last_frame = Instant::now();
        let mut total_frames = 0usize;

        let mut layers = Vec::new();
        let mut text = Text::new((380, 700), (950, 160));
        let mut character_text = Text::new((380, 640), (900, 50));
        let mut face_layer = Layer::default();

        text.use_cursor = true;

        layers.resize_with(30, Layer::default);

        let event_loop = self.event_loop.take().unwrap();

        let (mut cur_x, mut cur_y) = (0.0, 0.0);
        let mut mouse_entered = false;

        self.vm.load_command_until_wait().unwrap();

        use std::time::Duration;
        use winit::event::StartCause;

        let mut last_time = Instant::now();
        let tick_per_frame = Duration::from_secs_f64(1.0 / 60.0);

        let mut tick_text = true;

        event_loop.run(move |event, _evt_loop, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;

                self.vm.request_draw();
                self.surface.window().request_redraw();
            }
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + tick_per_frame)
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                let now = Instant::now();
                let delta_time = (now - last_time).as_secs_f32();
                self.surface.window().request_redraw();
                self.vm.request_draw();

                self.vm.tick_animator();

                if tick_text {
                    text.cursor += delta_time * 20.0;
                }
                last_time = now;

                *control_flow = ControlFlow::WaitUntil(now + tick_per_frame);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorEntered { .. },
                ..
            } => {
                mouse_entered = true;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorLeft { .. },
                ..
            } => {
                mouse_entered = false;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                cur_x = 1600.0 * position.x / self.surface.window().inner_size().width as f64;
                cur_y = 900.0 * position.y / self.surface.window().inner_size().height as f64;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                log::debug!("key press: {:?}", input);
            }
            Event::DeviceEvent {
                device_id: _,
                event,
            } => {
                use winit::event::{DeviceEvent, ElementState};

                match event {
                    DeviceEvent::Button {
                        state: ElementState::Pressed,
                        ..
                    } if mouse_entered => {
                        log::debug!("mouse down: ({:.1}, {:.1})", cur_x, cur_y);
                    }
                    DeviceEvent::Button {
                        state: ElementState::Released,
                        ..
                    } if mouse_entered => {
                        log::debug!("mouse up: ({:.1}, {:.1})", cur_x, cur_y);

                        self.vm.load_command_until_wait().unwrap();
                        self.surface.window().request_redraw();
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                // *control_flow = ControlFlow::Wait;
                tick_text = true;

                // TODO:
                if !self.vm.draw_requested {
                    // log::debug!("entering wait mode");
                    return;
                }

                let now = Instant::now();
                let commands = self.vm.poll();

                if total_frames > 30 {
                    log::debug!(
                        "fps: {:.2}",
                        (total_frames as f64) / (now - last_frame).as_secs_f64()
                    );
                    total_frames = 1;
                    last_frame = now;
                } else {
                    total_frames += 1;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if !commands.is_empty() {
                    log::debug!("commands received: {:?}", commands);
                }

                for c in commands {
                    match c {
                        DrawCall::LayerClear { layer } => {
                            log::debug!("layer clear");
                            layers[layer as usize].clear_layers();
                        }
                        DrawCall::LayerMoveTo {
                            layer,
                            origin: (x, y),
                        } => {
                            layers[layer as usize].move_to(x, y);
                        }
                        DrawCall::LayerLoadS25 { layer, path } => {
                            log::debug!("load_s25: {}", layer);

                            layers[layer as usize].load_s25(path).unwrap();
                        }
                        DrawCall::LayerSetCharacter { layer, pict_layers } => {
                            log::debug!("load_entries: {}", layer);

                            layers[layer as usize].load_pict_layers(&pict_layers);
                        }
                        DrawCall::Dialogue {
                            character_name,
                            dialogue,
                        } => {
                            log::debug!("dialogue: {}, character: {:?}", dialogue, character_name);
                            text.write(&dialogue, self.graphical_queue.clone());
                            text.load_gpu(self.graphical_queue.clone(), pipeline_text.clone());
                            text.cursor = 0.0;

                            if let Some(character_name) = character_name {
                                character_text.write(&character_name, self.graphical_queue.clone());
                                character_text
                                    .load_gpu(self.graphical_queue.clone(), pipeline_text.clone());
                            } else {
                                character_text.clear();
                            }

                            tick_text = false;
                        }
                        DrawCall::FaceLayerClear => {
                            log::debug!("face clear");
                            face_layer.clear_layers();
                        }
                        DrawCall::FaceLayerLoadS25 { path } => {
                            log::debug!("face load_s25");
                            face_layer.load_s25(path).unwrap();
                        }
                        DrawCall::FaceLayerSetCharacter { pict_layers } => {
                            log::debug!("face load_entries");
                            face_layer.load_pict_layers(&pict_layers);
                            face_layer.load_pict_layers_to_gpu(
                                self.graphical_queue.clone(),
                                pipeline.clone(),
                            );
                        } // _ => {}
                    }
                }

                for l in &mut layers {
                    l.load_pict_layers_to_gpu(self.graphical_queue.clone(), pipeline.clone());

                    if l.is_cached() {
                        continue;
                    }

                    previous_frame_end =
                        Some(Box::new(l.join_future(previous_frame_end.take().unwrap())));
                }

                if !face_layer.is_cached() {
                    previous_frame_end = Some(Box::new(
                        face_layer.join_future(previous_frame_end.take().unwrap()),
                    ));
                }

                if !text.is_cached() {
                    previous_frame_end = Some(Box::new(
                        text.join_future(previous_frame_end.take().unwrap()),
                    ));
                }

                if !character_text.is_cached() {
                    previous_frame_end = Some(Box::new(
                        character_text.join_future(previous_frame_end.take().unwrap()),
                    ));
                }

                if recreate_swapchain {
                    // Get the new dimensions of the window.
                    let dimensions: [u32; 2] = self.surface.window().inner_size().into();
                    let (new_swapchain, new_images) =
                        match self.swapchain.recreate_with_dimensions(dimensions) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    self.swapchain = new_swapchain;

                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut dynamic_state,
                    );
                    recreate_swapchain = false;
                }

                let (image_num, suboptimal, acquire_future) =
                    match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
                    self.device.clone(),
                    self.graphical_queue.family(),
                )
                .unwrap()
                .begin_render_pass(
                    framebuffers[image_num].clone(),
                    false,
                    vec![[1.0, 1.0, 1.0, 1.0].into()],
                )
                .unwrap();

                let mut command_buffer = command_buffer;
                for l in &mut layers {
                    command_buffer = l.draw(command_buffer, pipeline.clone(), &dynamic_state);
                }

                let command_buffer =
                    face_layer.draw(command_buffer, pipeline.clone(), &dynamic_state);

                let command_buffer =
                    text.draw(command_buffer, pipeline_text.clone(), &dynamic_state);

                let command_buffer =
                    character_text.draw(command_buffer, pipeline_text.clone(), &dynamic_state);

                let command_buffer = command_buffer.end_render_pass().unwrap().build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(self.graphical_queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        self.graphical_queue.clone(),
                        self.swapchain.clone(),
                        image_num,
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(Box::new(future) as Box<_>);
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end =
                            Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end =
                            Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
                    }
                }
            }
            _ => {
                // do nothing
            }
        });
    }

    pub fn perform_redraw(&mut self) {
        // TODo:
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
