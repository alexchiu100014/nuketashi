pub mod layer;

use std::path::PathBuf;

use layer::LayerModel;

#[derive(Clone, Default)]
pub struct FaceModel {
    pub filename: Option<PathBuf>,
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
            layers: (0_i32..30).map(|i| LayerModel::new(i)).collect(),
            ..Default::default()
        }
    }
}
