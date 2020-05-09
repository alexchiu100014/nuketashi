use crate::state::{EventPool, State};
use piston_window::*;
use std::f64;

use crate::s25::S25Archive;
use std::io::{BufRead, BufReader, Cursor, Read, Seek};

use std::path::PathBuf;

pub struct Hello<R> {
    script: BufReader<R>,
    is_waiting: bool,
    is_ctrl_pressing: bool,
    layers: Vec<Layer>,
    face: Option<Layer>,
    character_name: Option<String>,
    dialogue: Option<String>,
    font: Glyphs,
}

#[derive(Default)]
pub struct Layer {
    s25: Option<S25Archive>,
    global_origin: (i32, i32),
    textures: Vec<Option<G2dTexture>>,
    origin: Vec<(i32, i32)>,
}

fn find_asset(filename: &str) -> PathBuf {
    for entry in std::fs::read_dir("./blob/").unwrap() {
        let entry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            let mut file = entry.path().to_path_buf();
            file.push(filename);

            if file.exists() {
                return file;
            }
        }
    }

    unreachable!("missing file: {}; which was not expected", filename)
}

impl Hello<std::io::Cursor<String>> {
    pub fn new(font: Glyphs) -> Self {
        use encoding_rs::SHIFT_JIS;

        let inner = &include_bytes!("../../blob/NUKITASHI_T.WAR/01_C_01.TXT")[..];
        let (inner, _, _) = SHIFT_JIS.decode(inner);
        let inner = Cursor::new(inner.to_string());

        Hello {
            script: BufReader::new(inner),
            is_waiting: false,
            is_ctrl_pressing: false,
            layers: {
                let mut layers = Vec::new();
                layers.resize_with(30, Default::default);
                layers
            },
            face: None,
            character_name: None,
            dialogue: None,
            font,
        }
    }
}

impl<T> Hello<T> {
    fn exec_command(&mut self, command: &[&str]) {
        match command[0] {
            "$L_CHR" => {
                let layer_no: usize = command[1].parse().unwrap();

                if command.len() == 6 {
                    // add layer
                    let filename: &str = command[2].split('\\').skip(1).next().unwrap();
                    let x: i32 = command[3].parse().unwrap();
                    let y: i32 = command[4].parse().unwrap();
                    let entry: i32 = command[5].parse().unwrap();

                    if filename == "emo_0_0.s25" {
                        log::warn!("emo_0_0.s25 accompanied by $MOTION command");
                        log::warn!("the image is skipped; there is no way to display this");
                        log::warn!("visit https://github.com/3c1y/nkts for more information.");

                        return;
                    }

                    let mut arc = S25Archive::open(find_asset(filename)).unwrap();
                    let img = arc.load_image(entry as usize).unwrap();

                    let tex = crate::assets::load_image_from_rgba(
                        &img.rgba_buffer,
                        img.metadata.width as usize,
                        img.metadata.height as usize,
                        "___",
                    );

                    self.layers[layer_no] = Layer {
                        s25: Some(arc),
                        global_origin: (x, y),
                        textures: vec![Some(tex)],
                        origin: vec![(img.metadata.offset_x, img.metadata.offset_y)],
                    };
                } else {
                    // clear layer
                    self.layers[layer_no] = Layer::default();
                }
            }
            "$L_MONT" => {
                let layer_no: usize = command[1].parse().unwrap();

                let filename: &str = command[2].split('\\').skip(1).next().unwrap();
                let x: i32 = command[3].parse().unwrap();
                let y: i32 = command[4].parse().unwrap();

                if filename == "emo_0_0.s25" {
                    log::warn!("emo_0_0.s25 accompanied by $MOTION command");
                    log::warn!("the image is skipped; there is no way to display this");
                    log::warn!("visit https://github.com/3c1y/nkts for more information.");

                    return;
                }

                let mut arc = S25Archive::open(find_asset(filename)).unwrap();

                let mut textures = Vec::new();
                let mut origin = Vec::new();

                assert_eq!(command[6], "m");

                for (i, entry) in (&command[7..]).iter().enumerate() {
                    let entry: i32 = entry.parse().unwrap();

                    if entry == -1 {
                        textures.push(None);
                        origin.push((x, y));

                        continue;
                    }

                    let img = arc.load_image(entry as usize + 100 * i).unwrap();
                    let tex = crate::assets::load_image_from_rgba(
                        &img.rgba_buffer,
                        img.metadata.width as usize,
                        img.metadata.height as usize,
                        "___",
                    );

                    textures.push(Some(tex));
                    origin.push((img.metadata.offset_x, img.metadata.offset_y));
                }

                self.layers[layer_no] = Layer {
                    s25: Some(arc),
                    textures,
                    origin,
                    global_origin: (x, y),
                };
            }
            "$FACE" => {
                if command.len() == 1 {
                    self.face = None;
                    return;
                }

                let filename: &str = command[1].split('\\').skip(1).next().unwrap();
                let mut arc = S25Archive::open(find_asset(filename)).unwrap();

                let mut textures = Vec::new();
                let mut origin = Vec::new();

                assert_eq!(command[2], "m");

                for (i, entry) in (&command[3..]).iter().enumerate() {
                    let entry: i32 = entry.parse().unwrap();

                    if entry == -1 {
                        textures.push(None);
                        origin.push((0, 0));

                        continue;
                    }

                    let img = arc.load_image(entry as usize + 100 * i).unwrap();
                    let tex = crate::assets::load_image_from_rgba(
                        &img.rgba_buffer,
                        img.metadata.width as usize,
                        img.metadata.height as usize,
                        "___",
                    );

                    textures.push(Some(tex));
                    origin.push((img.metadata.offset_x, img.metadata.offset_y));
                }

                self.face = Some(Layer {
                    s25: Some(arc),
                    textures,
                    origin,
                    global_origin: (0, 0),
                });
            }
            "$A_CHR" => {
                // TODO: 
                let command_id: usize = command[1].parse().unwrap();

                match command_id {
                    150 => {
                        // dissolve?
                        let layer: usize = command[2].parse().unwrap();
                        let _duration: i32 = command[3].parse().unwrap();

                        self.layers[layer] = Layer {
                            s25: None,
                            textures: vec![],
                            origin: vec![],
                            global_origin: (0, 0)
                        };
                    }
                    128 => {
                        // moveBy?
                        let layer: usize = command[2].parse().unwrap();
                        let dx: i32 = command[2].parse().unwrap();
                        let dy: i32 = command[3].parse().unwrap();
                        let _duration: i32 = command[4].parse().unwrap();
                        let _unused: i32 = command[5].parse().unwrap();

                        for (x, y) in self.layers[layer].origin.iter_mut() {
                            *x += dx;
                            *y += dy;
                        }
                    }
                    _ => {
                        log::error!(
                            "unrecognized animator command {}; command with {} args",
                            command_id,
                            command.len() - 1
                        );
                    }
                }
            }
            _ => {
                log::error!("unrecognized command {}", command[0]);
            }
        }
    }

    fn display_message(&mut self, message: Vec<String>) {
        let mut character_name = None;
        let mut dialogue: String = String::new();

        for m in message {
            if m.starts_with('【') {
                character_name = Some(
                    m.trim_start_matches('【')
                        .trim_end_matches('】')
                        .split('/')
                        .last()
                        .unwrap()
                        .to_string(),
                );

                continue;
            }

            dialogue.push_str(&m);
        }

        self.character_name = character_name;
        self.dialogue = if dialogue.is_empty() {
            None
        } else {
            Some(dialogue)
        };
    }
}

impl<R> State for Hello<R>
where
    R: Read + Seek,
{
    fn draw(&mut self, v: (f64, f64), c: Context, g: &mut G2d) -> Option<()> {
        use piston_window::draw_state::Blend;

        // スクリーンバッファをクリア
        clear([1.0, 1.0, 1.0, 1.0], g);
        g.clear_stencil(0);

        let draw_state = c.draw_state.blend(Blend::Alpha);
        let transform = c.transform.scale(v.0 / 3200.0, v.1 / 1800.0);

        // 全てのレイヤーを描画
        for l in &self.layers {
            let transform = transform.trans(l.global_origin.0 as f64, l.global_origin.1 as f64);
            for (t, (x, y)) in l.textures.iter().zip(l.origin.iter()) {
                let transform = transform.trans(*x as f64, *y as f64);

                let t = if let Some(t) = t {
                    t
                } else {
                    continue;
                };

                Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(t, &draw_state, transform, g);
            }
        }

        // 顔を表示
        if let Some(l) = &self.face {
            for (t, (x, y)) in l.textures.iter().zip(l.origin.iter()) {
                let transform = transform.trans(*x as f64, *y as f64);

                let t = if let Some(t) = t {
                    t
                } else {
                    continue;
                };

                Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(t, &draw_state, transform, g);
            }
        }

        // 文字を表示
        if let Some(name) = &self.character_name {
            let transform = transform.trans(380.0, 680.0);

            // 影
            for i in 0..7 {
                for j in 0..7 {
                    let transform = transform.trans(i as f64 - 3.0, j as f64 - 3.0);

                    text::Text::new_color([0.0, 0.0, 0.0, 1.0], 32)
                        .draw(name, &mut self.font, &c.draw_state, transform, g)
                        .unwrap();
                }
            }

            // 中身
            text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32)
                .draw(name, &mut self.font, &c.draw_state, transform, g)
                .unwrap();
        }

        if let Some(dialogue) = &self.dialogue {
            let transform = transform.trans(380.0, 740.0);

            // 影
            for i in 0..7 {
                for j in 0..7 {
                    let transform = transform.trans(i as f64 - 3.0, j as f64 - 3.0);

                    text::Text::new_color([0.0, 0.0, 0.0, 1.0], 32)
                        .draw(dialogue, &mut self.font, &c.draw_state, transform, g)
                        .unwrap();
                }
            }

            // 中身
            text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32)
                .draw(dialogue, &mut self.font, &c.draw_state, transform, g)
                .unwrap();
        }

        Some(())
    }

    fn font_glyphs(&mut self) -> Option<&mut Glyphs> {
        Some(&mut self.font)
    }

    fn update(&mut self, event: &mut EventPool, _: f64) -> Option<()> {
        while let Some(evt) = event.poll_event() {
            use crate::state::GameEvent;

            match evt {
                GameEvent::MouseDown => {
                    log::debug!("Click!");
                    self.is_waiting = false;
                }
                GameEvent::KeyDown(key) => {
                    if key == Key::LCtrl.code() {
                        self.is_ctrl_pressing = true;
                    }
                }
                GameEvent::KeyUp(key) => {
                    if key == Key::LCtrl.code() {
                        self.is_ctrl_pressing = false;
                    }
                }
                _ => {}
            }
        }

        event.clear();

        if self.is_ctrl_pressing {
            self.is_waiting = false;
        }

        if self.is_waiting {
            return Some(());
        }

        // メッセージを進める
        let mut buf = String::new();
        let mut message_queue = Vec::new();

        while let Ok(size) = self.script.read_line(&mut buf) {
            if size == 0 {
                // log::debug!("end of scenario, congrats!");
                break;
            }

            let sbuf = buf.trim_end_matches(|p| p == '\n' || p == '\r');

            if sbuf.starts_with(";;") {
                // probably comment
                // do nothing here
            } else if sbuf.starts_with("$") {
                // command
                let command: Vec<_> = sbuf.split(',').collect();
                log::debug!("command {:?}", command);

                self.exec_command(&command);
            } else if sbuf.is_empty() && !message_queue.is_empty() {
                self.is_waiting = true;
                log::debug!("queue {:?}", message_queue);
                break;
            } else if !sbuf.is_empty() {
                message_queue.push(sbuf.to_string());
            }

            buf.clear();
        }

        self.display_message(message_queue);

        Some(())
    }
}
