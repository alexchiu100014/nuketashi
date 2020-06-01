use lazy_static::*;
use std::sync::Arc;
use vulkano::instance::{ApplicationInfo, Instance};

lazy_static! {
    // VkInstance should be created once.
    static ref VK_INSTANCE: Arc<Instance> = create_vulkan_instance();
}

#[inline(always)]
fn engine_name<'a>() -> String {
    let a = [237u8, 160, 133, 71, 47, 103, 14, 173, 49, 215, 83, 123, 188];
    let b = [191u8, 197, 236, 61, 74, 14, 103, 195, 101, 184, 38, 16, 221];

    let c: Vec<_> = a
        .iter()
        .copied()
        .zip(b.iter().copied())
        .map(|(a, b)| a ^ b)
        .collect();

    unsafe { std::str::from_utf8_unchecked(&c).into() }
}

fn application_info() -> ApplicationInfo<'static> {
    use vulkano::instance::Version;

    let engine_name = engine_name();

    ApplicationInfo {
        engine_name: Some(engine_name.into()),
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
        // disable the validation layer
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
