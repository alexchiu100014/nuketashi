pub mod layer;

use crate::script::vm::DrawCall;
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
            layers: vec![LayerModel::default(); 30],
            ..Default::default()
        }
    }

    pub fn send_draw_call(&mut self, call: &DrawCall) {
        match call {
            DrawCall::LayerClear { layer } => {
                self.layers[*layer as usize] = LayerModel::default();
            }
            DrawCall::LayerMoveTo {
                layer,
                origin: (x, y),
            } => {
                self.layers[*layer as usize].origin = (*x, *y);
            }
            DrawCall::LayerLoadS25 { layer, path } => {
                self.layers[*layer as usize].filename = Some(path.clone());
            }
            DrawCall::LayerSetCharacter { layer, pict_layers } => {
                self.layers[*layer as usize].entries = pict_layers.clone();
            }
            DrawCall::Dialogue {
                character_name,
                dialogue,
            } => {
                self.dialogue = dialogue.clone();
                self.character_name = character_name.clone();
            }
            DrawCall::FaceLayerClear => {
                self.face = Default::default();
            }
            DrawCall::FaceLayerLoadS25 { path } => {
                self.face.filename = Some(path.clone());
            }
            DrawCall::FaceLayerSetCharacter { pict_layers } => {
                self.face.entries = pict_layers.clone();
            } // _ => {}
        }
    }
}
