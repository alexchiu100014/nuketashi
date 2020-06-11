use super::command::Command;
use crate::script::mil::command::{
    Command as MilCommand, FaceEntry, LayerCommand, MmCommand, RendererCommand, RuntimeCommand,
    SavedataCommand,
};

#[derive(Clone, Debug, Default)]
pub struct Transpiler {
    commands: Vec<Command>,
    transpiled: Vec<MilCommand>,
}

impl Transpiler {
    pub fn new(commands: Vec<Command>) -> Self {
        Transpiler {
            commands,
            ..Default::default()
        }
    }

    pub fn transpile(mut self) -> Vec<MilCommand> {
        let mut commands = vec![];

        commands.append(&mut self.commands);

        // visit all commands and build command chunks
        for cmd in commands {
            match cmd {
                Command::Dialogue { character, text } => self.visit_dialogue(character, text),
                Command::LClear { layer } => self.visit_lclear(layer),
                Command::LChr {
                    layer,
                    filename,
                    x,
                    y,
                    entry,
                } => self.visit_lchr(layer, filename, x, y, entry),
                Command::LMont {
                    layer,
                    filename,
                    x,
                    y,
                    reserved,
                    entries,
                } => self.visit_lmont(layer, filename, x, y, reserved, entries),
                Command::LPriorityClear => self.visit_lpriorityclear(),
                Command::LPriority { priority } => self.visit_lpriority(priority),
                Command::Emotion { layer, filename } => self.visit_emotion(layer, filename),
                Command::Draw { duration } => self.visit_draw(duration),
                Command::DrawExEmpty { duration, unknown } => {
                    self.visit_draw_ex_empty(duration, unknown)
                }
                Command::DrawEx {
                    filename,
                    duration,
                    reserved_overlay_mode,
                } => self.visit_draw_ex(filename, duration, reserved_overlay_mode),
                Command::Ex { name, x, y } => self.visit_ex(name, x, y),
                Command::AChr { id, args } => self.visit_achr(id, args),
                Command::LDelay { layer, duration } => self.visit_ldelay(layer, duration),
                Command::LDelayAll { duration } => self.visit_ldelayall(duration),
                Command::FaceAuto { flag } => self.visit_faceauto(flag),
                Command::FaceAnime { flag } => self.visit_faceanime(flag),
                Command::FaceClear => self.visit_faceclear(),
                Command::Face { filename, entries } => self.visit_face(filename, entries),
                Command::Music {
                    filename,
                    is_looped,
                } => self.visit_music(filename, is_looped),
                Command::Voice { filename } => self.visit_voice(filename),
                Command::SE {
                    filename,
                    unknown,
                    channel,
                    reserved_delay,
                } => self.visit_se(filename, unknown, channel, reserved_delay),
                Command::MusicFade { duration } => self.visit_musicfade(duration),
                Command::SEFade { duration, channel } => self.visit_sefade(duration, channel),
                Command::Wait { duration } => self.visit_wait(duration),
                Command::Title { title } => self.visit_title(title),
                Command::RegMsg { unknown } => self.visit_regmsg(unknown),
                Command::StrFlag { unknown } => self.visit_strflag(unknown),
                Command::Window { unknown } => self.visit_window(unknown),
                Command::Label { unknown } => self.visit_label(unknown),
                Command::Movie {
                    filename,
                    unknown,
                    unknown_1,
                } => self.visit_movie(filename, unknown, unknown_1),
                Command::Effect { unknown, unknown_1 } => self.visit_effect(unknown, unknown_1),
                Command::GlEffect { unknown } => self.visit_gleffect(unknown),
                Command::Unknown => self.visit_unknown(),
                Command::Facet => self.visit_facet(),
            }
        }

        // populate prefetch commands

        self.transpiled
    }
}

impl Transpiler {
    fn send(&mut self, command: MilCommand) {
        self.transpiled.push(command);
    }
}

// visitors
impl Transpiler {
    fn visit_dialogue(&mut self, character: Option<String>, text: String) {
        // send dialogue
        self.send(MilCommand::RendererCommand(RendererCommand::Dialogue(
            character.clone(),
            text.clone(),
        )));

        // add entry to savedata
        self.send(MilCommand::SavedataCommand(SavedataCommand::AddLogEntry {
            name: character.clone(),
            face: None,
            text,
            voice: None,
        }));

        // wait until user event
        self.send(MilCommand::RuntimeCommand(RuntimeCommand::WaitUntilUserEvent));
    }

    fn visit_lclear(&mut self, layer: i32) {
        self.send(MilCommand::LayerCommand {
            layer_no: layer,
            command: LayerCommand::Unload,
        });
    }

    fn visit_lchr(&mut self, layer: i32, filename: String, x: f64, y: f64, entry: i32) {
        // load layer entries
        self.send(MilCommand::LayerCommand {
            layer_no: layer,
            command: LayerCommand::Load(filename, vec![entry]),
        });

        self.send(MilCommand::LayerCommand {
            layer_no: layer,
            command: LayerCommand::SetPosition(x, y),
        });
    }

    fn visit_lmont(
        &mut self,
        layer: i32,
        filename: String,
        x: f64,
        y: f64,
        _reserved: i32,
        entries: Vec<i32>,
    ) {
        self.send(MilCommand::LayerCommand {
            layer_no: layer,
            command: LayerCommand::Load(filename, entries),
        });

        self.send(MilCommand::LayerCommand {
            layer_no: layer,
            command: LayerCommand::SetPosition(x, y),
        });
    }

    fn visit_lpriorityclear(&mut self) {
        self.send(MilCommand::RendererCommand(
            RendererCommand::LayerPriorityClear,
        ));
    }

    fn visit_lpriority(&mut self, priority: Vec<i32>) {
        self.send(MilCommand::RendererCommand(RendererCommand::LayerPriority(
            priority,
        )));
    }

    fn visit_emotion(&mut self, layer: i32, filename: String) {
        log::error!("$EMOTION unimplemented");
    }

    fn visit_draw(&mut self, duration: f64) {
        self.send(MilCommand::RendererCommand(RendererCommand::Draw));
    }

    fn visit_draw_ex_empty(&mut self, duration: f64, unknown: f64) {
        log::error!("$DRAW_EX unimplemented");
    }

    fn visit_draw_ex(&mut self, filename: String, duration: f64, reserved_overlay_mode: i32) {
        log::error!("$DRAW_EX unimplemented");
    }

    fn visit_ex(&mut self, name: String, x: i32, y: i32) {
        log::error!("$EX unimplemented");
    }

    fn visit_achr(&mut self, id: i32, args: Vec<String>) {
        match id {
            _ => log::error!("unsupported animation: {}", id),
        }
    }

    fn visit_ldelay(&mut self, layer: i32, duration: f64) {
        self.send(MilCommand::LayerCommand {
            layer_no: layer,
            command: LayerCommand::LayerDelay(duration),
        });
    }

    fn visit_ldelayall(&mut self, duration: f64) {
        for layer_no in 0..crate::constants::TOTAL_LAYERS {
            self.send(MilCommand::LayerCommand {
                layer_no,
                command: LayerCommand::LayerDelay(duration),
            });
        }
    }

    fn visit_faceauto(&mut self, flag: bool) {
        // preprocessor
    }

    fn visit_faceanime(&mut self, flag: bool) {
        log::error!("face anime unimplemented");
    }

    fn visit_faceclear(&mut self) {
        self.send(MilCommand::RendererCommand(RendererCommand::ClearFace));
    }

    fn visit_face(&mut self, filename: String, entries: Vec<i32>) {
        self.send(MilCommand::RendererCommand(RendererCommand::PushFace(
            FaceEntry { filename, entries },
        )));
    }

    fn visit_music(&mut self, filename: String, is_looped: bool) {
        self.send(MilCommand::MmCommand(MmCommand::PlayMusic {
            filename,
            is_looped,
        }));
    }

    fn visit_voice(&mut self, filename: String) {
        self.send(MilCommand::MmCommand(MmCommand::PlayVoice(filename)));
    }

    fn visit_se(
        &mut self,
        filename: String,
        unknown: i32,
        channel: i32,
        reserved_delay: Option<f64>,
    ) {
        self.send(MilCommand::MmCommand(MmCommand::PlaySE(channel, filename)));
    }

    fn visit_musicfade(&mut self, duration: f64) {
        self.send(MilCommand::MmCommand(MmCommand::FadeMusic(duration)));
    }

    fn visit_sefade(&mut self, duration: f64, channel: i32) {
        self.send(MilCommand::MmCommand(MmCommand::FadeSE(channel, duration)));
    }

    fn visit_wait(&mut self, duration: f64) {
        self.send(MilCommand::RuntimeCommand(RuntimeCommand::Wait(duration)));
    }

    fn visit_title(&mut self, title: String) {
        // scenario metadata
        log::info!("scenario title: {}", title);
    }

    fn visit_regmsg(&mut self, unknown: i32) {
        log::error!("$REGMSG unimplemented");
    }

    fn visit_strflag(&mut self, unknown: i32) {
        // for larger text?
        log::error!("$STRFLAG unimplemented");
    }

    fn visit_window(&mut self, unknown: i32) {
        log::error!("$WINDOW not implemented");
    }

    fn visit_label(&mut self, unknown: i32) {
        log::error!("$WINDOW not implemented");
    }

    fn visit_movie(&mut self, filename: String, unknown: i32, unknown_1: i32) {
        self.send(MilCommand::MmCommand(MmCommand::PlayMovie(filename)));
    }

    fn visit_effect(&mut self, unknown: i32, unknown_1: Option<f64>) {
        log::error!("$WINDOW not implemented");
    }

    fn visit_gleffect(&mut self, unknown: Option<i32>) {
        log::error!("$WINDOW not implemented");
    }

    fn visit_unknown(&mut self) {}

    fn visit_facet(&mut self) {
        log::error!("$FACET not implemented");
    }
}
