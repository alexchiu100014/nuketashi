#![warn(clippy::all)]

pub mod config;
pub mod constants;
pub mod format;
pub mod game;
pub mod model;
pub mod platform;
pub mod renderer;
pub mod script;
pub mod utils;

fn main() {
    // #[cfg(debug_assertions)]
    {
        use simplelog::*;
        let logger = TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed);
        let _ = CombinedLogger::init(vec![logger]);
    }

    platform::setup_panic_handler();
    
    log::debug!("？？？「幾重にも辛酸を舐め、七難八苦を超え、艱難辛苦の果て、満願成就に至る——」");

    let game = game::Game::new();
    game.execute();
}
