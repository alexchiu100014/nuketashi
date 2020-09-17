#[cfg(target_os = "macos")]
pub mod macos;

pub fn setup_panic_handler()
{
    #[cfg(target_os = "macos")]
    macos::setup_panic_handler();
}
