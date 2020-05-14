#[derive(Clone, Default)]
pub struct LayerModel {
    pub filename: String,
    pub entries: Vec<i32>,
    pub origin: (i32, i32),
    pub opacity: f32,
    pub blur_radius: (i32, i32),
}

#[derive(Clone, Default)]
pub struct FaceModel {
    pub filename: Option<String>,
    pub entries: Vec<i32>,
}

#[derive(Clone, Default)]
pub struct GameState {
    pub layers: Vec<LayerModel>,
    pub face: FaceModel,
    pub character_name: Option<String>,
    pub dialogue: String,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            layers: vec![LayerModel::default(); 30],
            ..Default::default()
        }
    }
}
