pub mod s25;
pub mod script;

fn main() {
    #[cfg(debug_assertions)]
    {
        use simplelog::*;
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
        )
        .unwrap()])
        .unwrap();
    }

    log::debug!("？？？「幾重にも辛酸を舐め、七難八苦を超え、艱難辛苦の果て、満願成就に至る——」");
}
