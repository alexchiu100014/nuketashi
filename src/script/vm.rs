use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use crate::format::fautotbl;
use std::path::{Path, PathBuf};

use super::state::layer::LayerCommand;
use super::state::GameState;

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
    }, */
    // face layer
    FaceLayerClear,
    FaceLayerLoadS25 {
        path: PathBuf,
    },
    FaceLayerSetCharacter {
        pict_layers: Vec<i32>,
    },
    /* FaceAnimationEnable,
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
    /// Reader for the script file.
    pub reader: BufReader<R>,
    /// Buffer for draw calls. Flushed when $DRAW or $DRAW_EX called, or dialogue pushed.
    pub draw_calls: Vec<DrawCall>,
    /// A flag for redraw.
    pub draw_requested: bool,
    /// Queue for the layer effect.
    pub effect_queue: Vec<LayerEffect>,
    /// Face auto mode.
    pub face_auto_mode: bool,
    /// Face map.
    pub face_map: HashMap<String, String>,
    /// Face state cache.
    pub face_state_cache: HashMap<String, Vec<i32>>,
    /// Root directory for finding assets
    pub root_dir: PathBuf,
    /// Game state.
    pub state: GameState,
}

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
            face_map: Default::default(),
            face_state_cache: Default::default(),
            root_dir: "./blob/".into(),
            face_auto_mode: false,
            draw_requested: false,
            state: GameState::new(),
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

                    if self.face_auto_mode {
                        if let Some(n) = self.face_map.get(character_name) {
                            let n = n.to_ascii_uppercase();
                            if let Some(e) = self.face_state_cache.get(&n) {
                                let e = e.clone();
                                self.load_face(&format!("{}_01F.s25", n), e);
                            } else {
                                self.face_clear();
                            }
                        } else {
                            self.face_clear();
                        }
                    }

                    self.stop_all_animations();

                    self.send_draw_call(DrawCall::Dialogue {
                        character_name: Some(character_name.split('/').last().unwrap().into()),
                        dialogue: dialogue_buffer[1..].join(""),
                    });
                } else {
                    self.face_clear();
                    self.stop_all_animations();

                    self.send_draw_call(DrawCall::Dialogue {
                        character_name: None,
                        dialogue: dialogue_buffer.join(""),
                    });
                }

                self.request_draw();
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

// draw calls
impl<R> Vm<R> {
    pub fn poll(&mut self) -> Vec<DrawCall> {
        use std::time::Instant;

        // poll every layer events
        let mut draw_calls = vec![];
        let now = Instant::now();

        for l in &mut self.state.layers {
            l.poll(now, &mut draw_calls);
        }

        // take out all draw calls
        draw_calls.append(&mut self.draw_calls);

        if self.draw_requested {
            self.draw_requested = false;
            draw_calls
        } else {
            vec![]
        }
    }

    pub fn request_draw(&mut self) {
        self.draw_requested = true;
    }
}

// animator
impl<R> Vm<R> {
    pub fn tick_animator(&mut self) {
        // deprecated.
    }

    pub fn stop_all_animations(&mut self) {
        use std::time::Instant;
        let now = Instant::now();

        for l in &mut self.state.layers {
            l.finalize();
            l.poll(now, &mut self.draw_calls);
        }
    }
}

// face map and cache
impl<R> Vm<R> {
    pub fn construct_face_map<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let (face_filenames, face_names) = fautotbl::load_face_map(path)?;

        for (a, b) in face_filenames.into_iter().zip(face_names.into_iter()) {
            self.face_map.insert(b, a);
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

                    if filename == "emo_0_0.s25" {
                        log::warn!("emo_0_0.s25 accompanied by $MOTION command");
                        log::warn!("the image is skipped; there is no way to display this.");
                        log::warn!(
                            "this file is considered to be the initializer for the motion command."
                        );
                        log::warn!("visit https://github.com/3c1y/nkts for more information.");

                        return;
                    }

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
                    log::warn!("the image is skipped; there is no way to display this.");
                    log::warn!(
                        "this file is considered to be the initializer for the motion command."
                    );
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
            "$A_CHR" => {
                // animator command
                use super::state::layer::AnimationType;
                use crate::utils::easing::Easing;
                use std::time::Duration;

                match command[1].parse::<i32>().ok() {
                    Some(2) => {
                        // BOUNCE_Y
                        let layer: i32 = command[2].parse().unwrap();
                        let (n, dy): (i32, i32) =
                            (command[3].parse().unwrap(), command[4].parse().unwrap());
                        let msecs: f64 = command[5].parse().unwrap();

                        for _ in 0..n {
                            self.state.layers[layer as usize].send(LayerCommand::LayerAnimate {
                                duration: Duration::from_secs_f64(msecs * 1.0e-3),
                                easing: Easing::EaseOut,
                                to: AnimationType::MoveBy(0, dy),
                            });
                            self.state.layers[layer as usize].send(LayerCommand::LayerAnimate {
                                duration: Duration::from_secs_f64(msecs * 1.0e-3),
                                easing: Easing::EaseIn,
                                to: AnimationType::MoveBy(0, -dy),
                            });
                        }
                    }
                    Some(128) => {
                        // MOVE_TO
                        let layer: i32 = command[2].parse().unwrap();

                        let (x, y): (i32, i32) =
                            (command[3].parse().unwrap(), command[4].parse().unwrap());
                        let msecs: f64 = command[5].parse().unwrap();
                        let easing = match command[6].parse::<i32>().unwrap() {
                            1 => Easing::EaseOut,
                            _ => Easing::Linear,
                        };

                        self.state.layers[layer as usize].send(LayerCommand::LayerAnimate {
                            duration: Duration::from_secs_f64(msecs * 1.0e-3),
                            easing,
                            to: AnimationType::MoveTo(x, y),
                        });
                    }
                    Some(150) => {
                        // FADE_OUT
                        let layer: i32 = command[2].parse().unwrap();
                        let msecs: f64 = command[3].parse().unwrap();

                        self.state.layers[layer as usize].send(LayerCommand::LayerAnimate {
                            duration: Duration::from_secs_f64(msecs * 1.0e-3),
                            easing: Easing::EaseOut,
                            to: AnimationType::Opacity(0.0),
                        });
                        self.state.layers[layer as usize].send(LayerCommand::LayerClear);
                    }
                    Some(151) => {
                        // FADE_IN
                    }
                    _ => {
                        // unknown animation command
                    }
                }
            }
            "$L_DELAY" => {
                use std::time::Duration;

                assert_eq!(command[2], "T");
                let layer: usize = command[1].parse().unwrap();
                let msecs: f64 = command[3].parse().unwrap();

                self.state.layers[layer].send(LayerCommand::LayerDelay(Duration::from_secs_f64(
                    msecs * 1.0e-3,
                )));
            }
            "$FACE_AUTO" => {
                self.face_state_cache.clear();
                self.face_auto_mode = command[1] != "0";
            }
            "$FACE" => {
                if command.len() == 1 {
                    self.face_clear();
                    self.send_draw_call(DrawCall::Dialogue {
                        character_name: None,
                        dialogue: "".into(),
                    });
                } else {
                    let filename: &str = command[1].split('\\').skip(1).next().unwrap();

                    assert_eq!(command[2], "m");

                    self.load_face(
                        filename,
                        (&command[3..])
                            .iter()
                            .map(|e| e.parse::<i32>().unwrap_or(-1))
                            .collect(),
                    );
                }
            }
            _ => {}
        }
    }

    fn send_draw_call(&mut self, call: DrawCall) {
        log::debug!("draw call: {:?}", call);

        // queue for graphics engine
        self.draw_calls.push(call);
    }

    fn load_face(&mut self, filename: &str, entries: Vec<i32>) {
        // cache face
        self.face_state_cache
            .insert(filename.split('_').next().unwrap().into(), entries.clone());

        // send draw command
        self.send_draw_call(DrawCall::FaceLayerLoadS25 {
            path: self.lookup(filename),
        });

        self.send_draw_call(DrawCall::FaceLayerSetCharacter {
            pict_layers: entries,
        });
    }

    fn face_clear(&mut self) {
        self.send_draw_call(DrawCall::FaceLayerClear);
    }

    fn l_clear(&mut self, layer: i32) {
        let layer = &mut self.state.layers[layer as usize];
        layer.send(LayerCommand::LayerClear);
    }

    fn l_chr(&mut self, layer: i32, filename: &str, x: i32, y: i32, entry: i32) {
        let filename = self.lookup(filename);
        let layer = &mut self.state.layers[layer as usize];

        layer.send(LayerCommand::LayerLoadS25(filename));
        layer.send(LayerCommand::LayerLoadEntries(vec![entry]));
        layer.send(LayerCommand::LayerMoveTo(x, y));
        layer.send(LayerCommand::LayerWaitDraw);
    }

    fn l_mont(&mut self, layer: i32, filename: &str, x: i32, y: i32, entries: Vec<i32>) {
        // cache face
        self.face_state_cache
            .insert(filename.split('_').next().unwrap().into(), entries.clone());

        // send draw command
        let filename = self.lookup(filename);
        let layer = &mut self.state.layers[layer as usize];

        layer.send(LayerCommand::LayerLoadS25(filename));
        layer.send(LayerCommand::LayerLoadEntries(entries));
        layer.send(LayerCommand::LayerMoveTo(x, y));
        layer.send(LayerCommand::LayerWaitDraw);
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
