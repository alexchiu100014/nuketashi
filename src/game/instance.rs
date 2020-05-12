use lazy_static::*;
use std::sync::Arc;
use vulkano::instance::{ApplicationInfo, Instance};

use crate::constants;

lazy_static! {
    // VkInstance should be created once.
    static ref VK_INSTANCE: Arc<Instance> = create_vulkan_instance();
}

fn application_info() -> ApplicationInfo<'static> {
    use vulkano::instance::Version;

    ApplicationInfo {
        // ReizeiinTouka; ShiinaRio-script compatible engine.
        engine_name: Some(constants::GAME_ENGINE_NAME.into()),
        engine_version: Some(Version {
            major: 2,
            minor: 50,
            patch: 0,
        }),
        ..vulkano::app_info_from_cargo_toml!()
    }
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

pub fn get_instance() -> &'static Arc<Instance> {
    // Create an Vulkan instance by dereferencing VK_INSTANCE,
    // which is 'static (using lazy_static!).

    &*VK_INSTANCE
}
