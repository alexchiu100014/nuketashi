//! Structure representations of nkts commands.

pub struct LMont {
    pub layer: i32,
    pub path: String,
    pub x: i32,
    pub y: i32,
    _reserved: u32, // to be 0?
    pub pict_layers: Vec<Option<u32>>,
}

pub struct LChr {
    pub layer: i32,
    pub path: Option<String>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub entry_no: Option<i32>,
}

// 2: $A_CHR,00,2                           AChr(N1, L)                       clearing the anim?
// 3: $A_CHR,150,16,300                     AChr(N1, L, D)                    fade-out?
// 4: $A_CHR,60,1,c\eff_008.S25,300         AChr(N1, L, P, D)                 idk
// 5: $A_CHR,02,3,1,25,400                  AChr(N1, L, X, Y, D)              move
// 6: $A_CHR,128,15,-50,0,3000,1            AChr(N1, L, X, Y, D, N2)          move?
// 7: $A_CHR,20,13,140,488,360,3000,1       AChr(N1, L, L2, X, Y, D, N2)      idk
// 8: $A_CHR,06,1,4,15,100$L_DELAY,0,T,800                                    all acc. with $L_DELAY, probably = 5?
pub struct AChr {
    pub command_id: i32,
    pub rest: Vec<String>,
}

pub struct LDelay {
    // TODO:
}

// Flush all draw calls and fade into the next frame.
pub struct Draw {
    pub duration: i32,
}

// DrawEx(N1, P, D, F)
// Flush all draw calls and fade into the next frame
// with overlay.
pub struct DrawEx {
    pub num1: i32,            // 0 or 2, blending option?
    pub path: Option<String>, // .S25 file. optional
    pub duration: i32,        // duration of effect?
    pub flag: i32,            // 0 or 1,
}

pub struct Wait {
    pub duration: i32,
}

// $EX,PPFGBLUR,11,0
pub struct Effect {
    pub effect_name: String, // PPFGBLUR is only known
    pub blur_x: i32,
    pub blur_y: i32,
}

// $SE
pub struct SoundEffect {
    // TODO:
}

// $STR_FLAG
pub struct StrFlag {}

// $TITLE
pub struct Title {}

// $VOICE
pub struct Voice {
    // TODO:
}

// $MUSIC
pub struct Music {
    // TODO:
}

// $MOVIE
pub struct Movie {
    // TODO:
}

// $REGMSG
pub struct Regmsg {
    // TODO:
}

// $L_PRIORITY
pub struct LPriority {
    // TODO:
}

// $FACE
pub struct Face {
    // TODO:
}

// $GLEFFECT
pub struct GLEffect {
    // TODO:
}
// 【桐香/？？？】
pub struct DialogueHeader {
    pub name: String,
    pub alias: Option<String>,
}

// 「確かにそのバイブは強力です。しかし指先を狙えば無力化することなど造作もない」
pub struct Dialogue {
    pub name: String,
    pub alias: Option<String>,
}

// ;;○カットイン（淳之介）
// ;;01_c01へ
// ;;▽奈々瀬
// ;;★背景：熱帯林（夜） ;;★アイキャッチ ;;★「橘くん、前にあそこで会ったことあるよね？」
// ;;☆麻沙音_M_23_中央
// ;;●SE:椅子を引く（家）
// ;;「――このまま倒れるわけにはいかない」→05_fk_01へ
pub struct Miscellaneous {
    // おそらくコメント？
    pub is_comment: bool,
    pub command: String,
}
