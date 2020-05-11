use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use std::path::{Path, PathBuf};
use crate::format::fautotbl;

// Draw calls that will be sent to the graphics engine
#[derive(Debug, Clone)]
pub enum DrawCall {
    // layer calls
    LayerClear {
        layer: i32,
    },
    LayerClearOverlay {
        layer: i32,
    },
    LayerMoveTo {
        layer: i32,
        origin: (i32, i32),
    },
    LayerOpacity {
        layer: i32,
        opacity: f32,
    },
    LayerOverlayRate {
        layer: i32,
        rate: f32,
    },
    LayerLoadS25 {
        layer: i32,
        path: PathBuf,
    },
    LayerSetCharacter {
        layer: i32,
        pict_layers: Vec<u32>,
    },
    LayerLoadSingle {
        layer: i32,
        path: PathBuf,
        pict_layer: u32,
    },
    LayerLoadOverlay {
        layer: i32,
        path: PathBuf,
        pict_layer: u32,
    },
    // face layer
    FaceLayerClear {
        layer: i32,
    },
    FaceLayerLoadS25 {
        layer: i32,
        path: PathBuf,
    },
    FaceLayerSetCharacter {
        layer: i32,
        pict_layers: Vec<u32>,
    },
    FaceAnimationEnable,
    FaceAnimationDisable,
    // fade-overlay
    PushToFadeOverlay,
    ShowFadeOverlay {
        opacity: f32,
    },
    PopFadeOverlay,
}

#[derive(Clone, Copy)]
pub enum LayerEffect {
    PpfgBlur { radius: (i32, i32) },
}

pub struct Vm<R> {
    pub reader: BufReader<R>,
    pub draw_calls: Vec<DrawCall>,
    pub effect_queue: Vec<LayerEffect>,
    pub face_map: HashMap<String, String>,
}

// script parser
impl<R> Vm<R>
where
    R: Read,
{
    pub fn load_command_until_wait(&mut self) -> std::io::Result<()> {
        let mut buf = String::new();
        let mut dialogue_buffer: Vec<String> = vec![];

        loop {
            if self.reader.read_line(&mut buf)? == 0 {
                // end-of-line
                break;
            }

            let cmd = buf.trim_end_matches(|p| p == '\n' || p == '\r');

            if cmd.is_empty() && !dialogue_buffer.is_empty() {
                // flush draw command and display the dialouge
            }

            if cmd.starts_with('$') {
                // run command
                self.visit_command(cmd);
            } else if !cmd.starts_with(';') {
                // queue dialogue
                dialogue_buffer.push(cmd.to_owned());
            }

            buf.clear();
        }

        Ok(())
    }
}

// face map and cache
impl<R> Vm<R> {
    pub fn construct_face_map<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let (face_filenames, face_names) = fautotbl::load_face_map(path)?;

        for (a, b) in face_filenames.into_iter().zip(face_names.into_iter()) {
            self.face_map.insert(a, b);
        }

        Ok(())
    }
}

// command visitor
impl<R> Vm<R> {
    fn visit_command(&mut self, command: &str) {
        let command: Vec<_> = command.split(',').collect();

        match command[0] {
            "$FACE" => {}
            _ => {}
        }
    }

    fn _send_draw_call(&mut self, call: DrawCall) {
        println!("{:?}", call);
    }
}
