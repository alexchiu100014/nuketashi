// TODO: refactor

pub mod instance;
pub mod layer;
pub mod pipeline;
pub mod renderer;
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
use crate::script::vm::Vm;

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
            let script = include_bytes!("../../blob/NUKITASHI_T.WAR/01_C_00.TXT");
            let (script, _, _) = SHIFT_JIS.decode(script);
            // let script = include_str!("../../blob/___t.WAR/01_C_01.copy.TXT");
            script.into()
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
        use vulkano::swapchain::{SwapchainCreationError};

        let render_pass =
            pipeline::create_render_pass(self.device.clone(), self.swapchain.format());

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
        let mut recreate_swapchain = false;

        let event_loop = self.event_loop.take().unwrap();

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
            Event::RedrawRequested(_) => {
                if recreate_swapchain {
                    // Get the new dimensions of the window.
                    let dimensions: [u32; 2] = self.surface.window().inner_size().into();
                    let (new_swapchain, new_images) =
                        match self.swapchain.recreate_with_dimensions(dimensions) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("failed to recreate swapchain: {:?}", e),
                        };

                    self.swapchain = new_swapchain;

                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut dynamic_state,
                    );
                    recreate_swapchain = false;
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
