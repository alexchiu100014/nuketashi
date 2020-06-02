#[derive(Clone, Debug)]
pub struct AnimationGraph {}

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
    PlayMusic(String),
    FadeSE(i32, f64),
    FadeMusic(f64),
}

#[derive(Clone, Debug)]
pub struct FaceEntry {
    filename: String,
    entries: Vec<i32>,
}

#[derive(Clone, Debug)]
pub enum SavedataCommand {
    AddLogEntry {
        name: Option<String>,
        face: FaceEntry,
        text: String,
        voice: Option<String>,
    },
    QuickSave,
    Save(i32),
}

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
}
