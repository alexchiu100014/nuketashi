pub mod s25;
pub mod script;

// swarm-game由来のコードをば
pub mod assets;
pub mod state;

fn main() {
    #[cfg(debug_assertions)]
    {
        // Loggerの割り当て
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

    use piston_window::*;

    let window_size = Size {
        width: 1600.0,
        height: 900.0,
    };
    let opengl_version = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("冷泉院桐香 v2.50", window_size)
        .exit_on_esc(true)
        .graphics_api(opengl_version)
        // .fullscreen(true)
        .build()
        .unwrap();

    // crate::assets (アセットを管理するモジュール) の初期化処理
    use crate::assets::set_texture_context;
    set_texture_context(window.create_texture_context());

    // Pistonのイベントループを回す。イベントはcrate::state::StateManagerが管理する。
    use crate::state::StateManager;

    let mut mgr = StateManager::new();
    // let test = mgr.set_state(crate::state::splash::Splash::new()); // 最初のシーン
    mgr.set_state(state::hello::Hello::new()); // debug

    while let Some(e) = window.next() {
        mgr.handle_event(&mut window, e);
    }
}
