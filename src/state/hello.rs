use crate::state::{EventPool, State};
use piston_window::*;
use std::f64;

use crate::s25::{S25Archive, S25Image};
use std::io::{BufRead, BufReader, Cursor, Read, Seek};

use std::path::PathBuf;

pub struct Hello<R> {
    script: BufReader<R>,
    is_waiting: bool,
    is_ctrl_pressing: bool,
    layers: Vec<Layer>,
    face: Option<Layer>,
}

#[derive(Default)]
pub struct Layer {
    s25: Option<S25Archive>,
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

impl Hello<Cursor<&'static [u8]>> {
    pub fn new() -> Self {
        Hello {
            script: BufReader::new(Cursor::new(
                &include_bytes!("../../blob/___t.WAR/02_NK_03.TXT")[..],
            )),
            is_waiting: false,
            is_ctrl_pressing: false,
            layers: {
                let mut layers = Vec::new();
                layers.resize_with(30, Default::default);
                layers
            },
            face: None,
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
                        textures: vec![Some(tex)],
                        origin: vec![(x + img.metadata.offset_x, y + img.metadata.offset_y)],
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
                    origin.push((img.metadata.offset_x + x, img.metadata.offset_y + y));
                }

                self.layers[layer_no] = Layer {
                    s25: Some(arc),
                    textures,
                    origin,
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
                });
            }
            _ => {
                log::error!("unrecognized command {}", command[0]);
            }
        }
    }
}

impl Default for Hello<Cursor<&'static [u8]>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> State for Hello<R>
where
    R: Read + Seek,
{
    fn draw(&mut self, _v: (f64, f64), c: Context, g: &mut G2d) -> Option<()> {
        use piston_window::draw_state::Blend;

        // スクリーンバッファをクリア
        clear([1.0, 1.0, 1.0, 1.0], g);
        g.clear_stencil(0);

        let draw_state = c.draw_state.blend(Blend::Alpha);

        // 全てのレイヤーを描画
        for l in &self.layers {
            for (t, (x, y)) in l.textures.iter().zip(l.origin.iter()) {
                let transform = c.transform.trans(*x as f64, *y as f64);

                let t = if let Some(t) = t {
                    t
                } else {
                    continue;
                };

                Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(t, &draw_state, transform, g);
            }
        }

        if let Some(l) = &self.face {
            for (t, (x, y)) in l.textures.iter().zip(l.origin.iter()) {
                let transform = c.transform.trans(*x as f64, *y as f64);

                let t = if let Some(t) = t {
                    t
                } else {
                    continue;
                };

                Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(t, &draw_state, transform, g);
            }
        }

        Some(())
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
                log::debug!("end of scenario, congrats!");
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

        Some(())
    }
}
