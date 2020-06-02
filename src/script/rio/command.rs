#[derive(Clone, PartialEq)]
pub enum Variable {
    String(String),
    Decimal(i32),
    Float(f64),
    Empty,
}

#[derive(Clone, PartialEq)]
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
    // Image manipulation
    Draw {
        duration: f64,
    },
    DrawExEmpty {
        // $DRAW_EX, 0, ..
        duration: f64,
        reserved_overlay_mode: i32,
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
        args: Vec<Variable>,
    },
    LDelay {
        layer: i32,
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
        voice: String,
    },
    SE {
        filename: String,
        unknown: i32,
        channel: i32,
        reserved_delay: Option<i32>,
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
}
