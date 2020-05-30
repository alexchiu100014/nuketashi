#![warn(clippy::all)]

pub mod constants;
pub mod format;
pub mod game;
pub mod platform;
pub mod renderer;
pub mod script;
pub mod utils;

fn main() {
    #[cfg(debug_assertions)]
    {
        use simplelog::*;

        if let Some(logger) =
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed)
        {
            // workaround for the OSX bundle
            let _ = CombinedLogger::init(vec![logger]);
        }
    }

    log::debug!("？？？「幾重にも辛酸を舐め、七難八苦を超え、艱難辛苦の果て、満願成就に至る——」");
}
