#[derive(Clone, PartialEq, Debug)]
pub enum Variable {
    String(String),
    Decimal(i32),
    Float(f64),
    Empty,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Command {
    // Dialogue
    Dialogue {
        character: Option<String>,
        text: String,
    },
    // Layer manipulation
    LClear {
        layer: i32,
    },
    LChr {
        layer: i32,
        filename: String,
        x: f64,
        y: f64,
        entry: i32,
    },
    LMont {
        layer: i32,
        filename: String,
        x: f64,
        y: f64,
        reserved: i32, // should be zero
        entries: Vec<i32>,
    },
    LPriorityClear,
    LPriority {
        priority: Vec<i32>,
    },
    Emotion {
        layer: i32,
        filename: String,
    },
    // Image manipulation
    Draw {
        duration: f64,
    },
    DrawExEmpty {
        // $DRAW_EX, 0, ..
        duration: f64,
        unknown: f64,
    },
    DrawEx {
        // $DRAW_EX, 2, ..
        filename: String,
        duration: f64,
        reserved_overlay_mode: i32, // 1?
    },
    Ex {
        name: String,
        x: i32,
        y: i32,
    },
    // Animation
    AChr {
        id: i32,
        args: Vec<String>,
    },
    LDelay {
        layer: i32,
        duration: f64,
    },
    LDelayAll {
        duration: f64,
    },
    // Face layer
    FaceAuto {
        flag: bool,
    },
    FaceAnime {
        flag: bool,
    },
    FaceClear,
    Face {
        filename: String,
        entries: Vec<i32>,
    },
    // Sound
    Music {
        filename: String,
        is_looped: bool,
    },
    Voice {
        filename: String,
    },
    SE {
        filename: String,
        unknown: i32,
        channel: i32,
        reserved_delay: Option<f64>,
    },
    MusicFade {
        duration: f64,
    },
    SEFade {
        duration: f64,
        channel: i32,
    },
    // Others
    Wait {
        duration: f64,
    },
    Title {
        title: String,
    },
    RegMsg {
        unknown: i32,
    },
    StrFlag {
        unknown: i32,
    },
    Window {
        unknown: i32,
    },
    Label {
        unknown: i32,
    },
    Movie {
        filename: String,
        unknown: i32,
        unknown_1: i32,
    },
    Effect {
        unknown: i32,
        unknown_1: Option<f64>,
    },
    GlEffect {
        unknown: Option<i32>,
    },
    Unknown,
    Facet,
}
