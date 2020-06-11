#[derive(Copy, Clone, Debug)]
pub enum AnimationTarget {
    OffsetX,
    OffsetY,
    OverlayRate,
    Opacity,
}

#[derive(Copy, Clone, Debug)]
pub struct AnimationStem {
    pub target: AnimationTarget,
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Debug)]
pub struct AnimationGraph {
    stems: Vec<AnimationStem>,
    duration: f64,
    delay: f64,
    then: Vec<Command>,
    finalize: Vec<Command>,
}

#[derive(Clone, Debug)]
pub enum LayerCommand {
    Load(String, Vec<i32>),
    Unload,
    Prefetch(String, Vec<i32>),
    SetPosition(f64, f64),
    SetOpacity(f64),
    SetBlurRate(i32, i32),
    LoadOverlay(String, i32, i32), // filename, entry, overlay mode
    UnloadOverlay,
    SetOverlayRate(f64),
    LoadAnimationGraph(AnimationGraph),
    WaitUntilAnimationIsDone,
    FinalizeAnimation,
    LayerDelay(f64),
}

#[derive(Clone, Debug)]
pub enum RendererCommand {
    LoadOverlay(String, i32, i32),
    UnloadOverlay,
    SetOverlayRate(f64),
    PushScreen,
    ClearFace,
    PushFace(FaceEntry),
    Dialogue(Option<String>, String),
    LayerPriorityClear,
    LayerPriority(Vec<i32>),
    Draw,
}

#[derive(Clone, Debug)]
pub enum RuntimeCommand {
    Wait(f64),
    WaitUntilUserEvent,
}

#[derive(Clone, Debug)]
pub enum MmCommand {
    PlayMovie(String),
    PlaySE(i32, String),
    PlayVoice(String),
    PlayMusic {
        filename: String,
        is_looped: bool,
    },
    FadeSE(i32, f64),
    FadeMusic(f64),
}

#[derive(Clone, Debug)]
pub struct FaceEntry {
    pub(crate) filename: String,
    pub(crate) entries: Vec<i32>,
}

#[derive(Clone, Debug)]
pub enum SavedataCommand {
    AddLogEntry {
        name: Option<String>,
        face: Option<FaceEntry>,
        text: String,
        voice: Option<String>,
    },
    QuickSave,
    QuickLoad,
    Save(i32),
    Load(i32),
    BackupSave,
    BackupLoadIfAvailable,
}

#[derive(Clone, Debug)]
pub enum PassCommand {
    FaceAuto(bool),
}

use crate::script::rio::command::Command as RioCommand;

#[derive(Clone, Debug)]
pub enum Command {
    LayerCommand {
        layer_no: i32,
        command: LayerCommand,
    },
    RendererCommand(RendererCommand),
    RuntimeCommand(RuntimeCommand),
    SavedataCommand(SavedataCommand),
    MmCommand(MmCommand),
    // for fallback
    UnsupportedCommand(RioCommand),
    // for passes
    PassCommand(PassCommand),
}
