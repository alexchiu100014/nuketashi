// vulkano; Vulkan rapper
use vulkano::instance::{ApplicationInfo, Instance, PhysicalDevice};

// required for the VkSurface creation
use vulkano_win::VkSurfaceBuild;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use lazy_static::*;
use std::sync::Arc;

pub struct Game<'a> {
    physical: PhysicalDevice<'a>,
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

        if layers.is_none() {
            log::warn!("validation layer not supported");
        } else {
            log::debug!("validation layer (VK_LAYER_KHRONOS_validation) supported and enabled");
        }
    }

    #[cfg(not(debug_assertions))]
    {
        layers = None;
    }

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
            minor: 5,
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

        let physical = Self::create_physical();

        Game { physical }
    }
}

impl<'a> Game<'a> {
    fn create_physical() -> PhysicalDevice<'static> {
        // Create an Vulkan instance by dereferencing VK_INSTANCE,
        // which is 'static (using lazy_static!).
        let instance = &*VK_INSTANCE;
        log::debug!("created Vulkan instance");

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

    /// Executes an event loop.
    ///
    /// It takes the ownership of a Game instance, and won't return until
    /// the program is closed.
    pub fn execute(self) {
        // TODO:
    }
}
