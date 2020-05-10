// vulkano; Vulkan rapper
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{ApplicationInfo, Instance, PhysicalDevice, QueueFamily};
use vulkano::swapchain::Surface;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use lazy_static::*;
use std::sync::Arc;

use crate::constants;

pub struct Game<'a> {
    physical: PhysicalDevice<'a>,
    event_loop: EventLoop<()>,
    surface: Arc<Surface<Window>>,
}

lazy_static! {
    static ref VK_INSTANCE: Arc<Instance> = create_vulkan_instance();
}

fn create_vulkan_instance() -> Arc<Instance> {
    let extensions = vulkano_win::required_extensions();

    #[cfg(debug_assertions)]
    let available_layers;
    let layers;

    #[cfg(debug_assertions)]
    {
        // create the validation layer
        available_layers = vulkano::instance::layers_list()
            .expect("failed to obtain supported layers")
            .find(|l| l.name() == "VK_LAYER_KHRONOS_validation");

        layers = available_layers.as_ref().map(|l| l.name());

        if let Some(l) = layers {
            log::debug!("validation layer ({}) supported and enabled", l);
        } else {
            log::warn!("validation layer not supported");
        }
    }

    #[cfg(not(debug_assertions))]
    {
        layers = None;
    }

    log::debug!("creating Vulkan instance");

    Instance::new(Some(&application_info()), &extensions, layers)
        .expect("failed to create Vulkan interface")
}

fn application_info() -> ApplicationInfo<'static> {
    use vulkano::instance::Version;

    ApplicationInfo {
        // ReizeiinTouka; ShiinaRio-script compatible engine.
        engine_name: Some("ReizeiinTouka".into()),
        engine_version: Some(Version {
            major: 2,
            minor: 50,
            patch: 0,
        }),
        ..vulkano::app_info_from_cargo_toml!()
    }
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
        let (device, queues) = Self::create_device(physical, &surface);

        Game {
            physical,
            event_loop,
            surface,
        }
    }
}

// -- Initialization
impl<'a> Game<'a> {
    fn create_physical() -> PhysicalDevice<'static> {
        // Create an Vulkan instance by dereferencing VK_INSTANCE,
        // which is 'static (using lazy_static!).
        let instance = &*VK_INSTANCE;

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
            .with_title(constants::GAME_ENGINE_NAME)
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

        let surface = vulkano_win::create_vk_surface(window, VK_INSTANCE.clone())
            .expect("failed to build Vulkan surface");

        log::debug!("created Vulkan surface");

        surface
    }

    fn create_device<T>(
        physical: PhysicalDevice,
        surface: &Surface<T>,
    ) -> (Arc<Device>, Arc<Queue>) {
        // graphical queue
        let gr_queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics())
            .expect("failed to find a graphical queue family");

        let pr_queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics()
                    && surface.is_supported(q).unwrap_or(false)
                    && gr_queue_family != q
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
            [(gr_queue_family, 1.0), (pr_queue_family, 0.5)]
                .iter()
                .cloned(),
        )
        .expect("failed to create device");

        // graphics queue and transfer queue
        let gq = q.next().unwrap();
        let pq = q.next().unwrap();

        log::debug!("created device and queue");

        (d, gq)
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
                _ => {
                    // do nothing
                }
            });
    }
}
