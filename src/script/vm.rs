use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use crate::format::fautotbl;
use std::path::{Path, PathBuf};

// Draw calls that will be sent to the graphics engine
#[derive(Debug, Clone)]
pub enum DrawCall {
    // layer calls
    LayerClear {
        layer: i32,
    },
    LayerMoveTo {
        layer: i32,
        origin: (i32, i32),
    },
    LayerLoadS25 {
        layer: i32,
        path: PathBuf,
    },
    LayerSetCharacter {
        layer: i32,
        pict_layers: Vec<i32>,
    },
    /* LayerOpacity {
        layer: i32,
        opacity: f32,
    },
    LayerOverlayRate {
        layer: i32,
        rate: f32,
    },
    LayerLoadOverlay {
        layer: i32,
        path: PathBuf,
        pict_layer: u32,
    },
    LayerClearOverlay {
        layer: i32,
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
    PopFadeOverlay, */
    // text
    Dialogue {
        character_name: Option<String>,
        dialogue: String,
    },
}

#[derive(Clone, Copy)]
pub enum LayerEffect {
    PpfgBlur { radius: (i32, i32) },
}

pub struct Vm<R> {
    pub reader: BufReader<R>,
    pub draw_calls: Vec<DrawCall>,
    pub draw_requested: bool,
    pub effect_queue: Vec<LayerEffect>,
    pub face_map: HashMap<String, String>,
    pub root_dir: PathBuf,
    pub animation: Vec<Animation>,
}

pub struct Animation {}

// constructor
impl<R> Vm<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            draw_calls: vec![],
            effect_queue: vec![],
            animation: vec![],
            face_map: Default::default(),
            root_dir: "./blob/".into(),
            draw_requested: false,
        }
    }
}

// script parser
impl<R> Vm<R>
where
    R: Read,
{
    pub fn load_command_until_wait(&mut self) -> std::io::Result<bool> {
        let mut buf = String::new();
        let mut dialogue_buffer: Vec<String> = vec![];

        loop {
            if self.reader.read_line(&mut buf)? == 0 {
                // end-of-file
                return Ok(false);
            }

            let cmd = buf.trim_end_matches(|p| p == '\n' || p == '\r');

            if cmd.is_empty() && !dialogue_buffer.is_empty() {
                // flush draw command and display the dialouge
                log::debug!("flush draw command: {:?}", self.draw_calls);
                log::debug!("flush dialogue: {:?}", dialogue_buffer);

                if dialogue_buffer
                    .get(0)
                    .and_then(|v| Some(v.starts_with("【")))
                    .unwrap_or(false)
                {
                    let character_name = dialogue_buffer[0]
                        .trim_start_matches("【")
                        .trim_end_matches("】");

                    self.send_draw_call(DrawCall::Dialogue {
                        character_name: Some(character_name.split('/').last().unwrap().into()),
                        dialogue: dialogue_buffer[1..].join(""),
                    });
                } else {
                    self.send_draw_call(DrawCall::Dialogue {
                        character_name: None,
                        dialogue: dialogue_buffer.join(""),
                    });
                }

                self.draw_requested = true;
                dialogue_buffer.clear();

                break;
            }

            if cmd.starts_with('$') {
                // run command
                self.visit_command(cmd);
            } else if !cmd.is_empty() && !cmd.starts_with(';') {
                // queue dialogue
                dialogue_buffer.push(cmd.to_owned());
            }

            buf.clear();
        }

        Ok(true)
    }
}

impl<R> Vm<R> {
    pub fn poll(&mut self) -> Vec<DrawCall> {
        if self.draw_requested {
            self.draw_requested = false;
            std::mem::replace(&mut self.draw_calls, vec![])
        } else {
            vec![]
        }
    }

    pub fn request_draw(&mut self) {
        self.draw_requested = true;
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

        log::debug!("command: {:?}", command);

        match command[0] {
            "$L_CHR" => {
                let layer_no: i32 = command[1].parse().unwrap();

                if command.len() == 6 {
                    let filename: &str = command[2].split('\\').skip(1).next().unwrap();
                    let x: i32 = command[3].parse().unwrap();
                    let y: i32 = command[4].parse().unwrap();
                    let entry: i32 = command[5].parse().unwrap();

                    self.l_chr(layer_no, filename, x, y, entry);
                } else {
                    self.l_clear(layer_no);
                }
            }
            "$L_MONT" => {
                let layer: i32 = command[1].parse().unwrap();

                let filename: &str = command[2].split('\\').skip(1).next().unwrap();
                let x: i32 = command[3].parse().unwrap();
                let y: i32 = command[4].parse().unwrap();

                if filename == "emo_0_0.s25" {
                    log::warn!("emo_0_0.s25 accompanied by $MOTION command");
                    log::warn!("the image is skipped; there is no way to display this");
                    log::warn!("visit https://github.com/3c1y/nkts for more information.");

                    return;
                }

                assert_eq!(command[6], "m");

                self.l_mont(
                    layer,
                    filename,
                    x,
                    y,
                    (&command[7..])
                        .iter()
                        .map(|e| e.parse::<i32>().unwrap_or(-1))
                        .collect(),
                );
            }
            "$DRAW" => {
                let _fade_duration: i32 = command[1].parse().unwrap();
                self.request_draw();
            }
            // "$FACE" => {}
            _ => {}
        }
    }

    fn send_draw_call(&mut self, call: DrawCall) {
        log::debug!("draw call: {:?}", call);
        self.draw_calls.push(call);
    }

    fn l_clear(&mut self, layer: i32) {
        self.send_draw_call(DrawCall::LayerClear { layer });
    }

    fn l_chr(&mut self, layer: i32, filename: &str, x: i32, y: i32, entry: i32) {
        self.send_draw_call(DrawCall::LayerLoadS25 {
            layer,
            path: self.lookup(filename),
        });

        self.send_draw_call(DrawCall::LayerSetCharacter {
            layer,
            pict_layers: vec![entry],
        });

        self.send_draw_call(DrawCall::LayerMoveTo {
            layer,
            origin: (x, y),
        });
    }

    fn l_mont(&mut self, layer: i32, filename: &str, x: i32, y: i32, entries: Vec<i32>) {
        self.send_draw_call(DrawCall::LayerLoadS25 {
            layer,
            path: self.lookup(filename),
        });

        self.send_draw_call(DrawCall::LayerSetCharacter {
            layer,
            pict_layers: entries,
        });

        self.send_draw_call(DrawCall::LayerMoveTo {
            layer,
            origin: (x, y),
        });
    }
}

// utils
impl<R> Vm<R> {
    fn lookup_into(&self, filename: &str, dir: &Path) -> Option<PathBuf> {
        for d in std::fs::read_dir(dir) {
            for e in d {
                if let Ok(entry) = e {
                    if entry.metadata().unwrap().is_dir() {
                        if let Some(r) = self.lookup_into(filename, &entry.path()) {
                            return Some(r);
                        }
                    }

                    let path = entry.path();
                    let entry_name = path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_ascii_uppercase();
                    let entry_stem = path
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_ascii_uppercase();

                    if entry_stem.ends_with("(1)")
                        && filename.starts_with(entry_stem.trim_end_matches("(1)"))
                    {
                        return Some(entry.path().into());
                    } else if entry_name == filename {
                        return Some(entry.path().into());
                    }
                }
            }
        }

        None
    }

    fn lookup(&self, filename: &str) -> PathBuf {
        self.lookup_into(&filename.to_ascii_uppercase(), &self.root_dir)
            .unwrap()
    }
}
