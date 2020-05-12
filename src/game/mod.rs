pub mod instance;
pub mod pipeline;
pub mod shaders;
pub mod text;
pub mod texture_loader;

// vulkano; Vulkan rapper
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{ApplicationInfo, Instance, PhysicalDevice};
use vulkano::swapchain::{
    ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform, Swapchain,
};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;

use crate::constants;

pub struct Game<'a> {
    physical: PhysicalDevice<'a>,
    event_loop: EventLoop<()>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
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

        let event_loop = EventLoop::new();

        #[cfg(target_os = "macos")]
        unsafe {
            // Create a menu-bar for macOS.
            crate::platform::macos::create_menu_bar();
        }

        let physical = Self::create_physical();
        let surface = Self::create_window(&event_loop);
        let (device, graphical_queue, transfer_queue) = Self::create_device(physical, &surface);

        let caps = surface.capabilities(physical).unwrap();

        let (swapchain, image) = Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            caps.supported_formats[0].0,
            surface.window().inner_size().into(),
            1,
            caps.supported_usage_flags,
            &graphical_queue,
            SurfaceTransform::Identity,
            caps.supported_composite_alpha.iter().next().unwrap(),
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .expect("failed to create a swapchain");

        Game {
            physical,
            event_loop,
            surface,
            swapchain,
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
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .expect("failed to find a graphical queue family");

        let tr_queue_family = physical
            .queue_families()
            .find(|&q| {
                (q.supports_graphics() || q.supports_compute()) // VK_QUEUE_TRANSFER_BIT
                && gr_queue_family != q // no overlap
            })
            .expect("failed to find a presentation queue family");

        let extensions = DeviceExtensions {
            khr_swapchain: true, // swapchain is required
            ..DeviceExtensions::none()
        };

        let (d, mut q) = Device::new(
            physical,
            physical.supported_features(),
            &extensions,
            [(gr_queue_family, 1.0), (tr_queue_family, 0.5)]
                .iter()
                .cloned(),
        )
        .expect("failed to create device");

        // graphics queue and transfer queue
        let gq = q.next().unwrap();
        let tq = q.next().unwrap();

        log::debug!("created device and queue");

        (d, gq, tq)
    }
}

// -- Run-loop execution & event-handling
impl<'a> Game<'a> {
    /// Executes an event loop.
    ///
    /// It takes the ownership of a Game instance, and won't return until
    /// the program is closed.
    pub fn execute(self) {
        self.event_loop
            .run(move |event, _evt_loop, control_flow| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::RedrawRequested(_) => {
                    // self.perform_draw();
                }
                _ => {
                    // do nothing
                }
            });
    }
}
