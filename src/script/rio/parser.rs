//! Parser for ShiinaRio script

use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek};
use std::path::Path;

use super::command::Command;

pub struct Parser<R> {
    reader: BufReader<R>,
}

impl Parser<File> {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self {
            reader: BufReader::new(File::open(path)?),
        })
    }
}

impl Parser<()> {
    pub fn from_raw_bytes(bytes: &[u8]) -> Parser<Cursor<&[u8]>> {
        Parser {
            reader: BufReader::new(Cursor::new(bytes)),
        }
    }

    pub fn from_raw_bytes_owned(bytes: Vec<u8>) -> Parser<Cursor<Vec<u8>>> {
        Parser {
            reader: BufReader::new(Cursor::new(bytes)),
        }
    }

    pub fn from_reader<R>(reader: R) -> Parser<R>
    where
        R: Read + Seek,
    {
        Parser {
            reader: BufReader::new(reader),
        }
    }
}

impl<R> Parser<R>
where
    R: Read + Seek,
{
    pub fn parse(&mut self) -> std::io::Result<Vec<Command>> {
        let mut buf = String::new();
        let mut dialogue_buffer: Vec<String> = vec![];
        let mut commands = Vec::new();

        'l: loop {
            // make sure that the buffer is clear
            buf.clear();

            if self.reader.read_line(&mut buf)? == 0 {
                break 'l;
            }

            let cmd = buf.trim_end_matches(|p| p == '\n' || p == '\r');

            if cmd.starts_with('$') {
                // it's a command!
                let cmd: String = cmd.trim_end().chars().filter(|c| c.is_ascii()).collect();
                commands.push(self.visit_command(&cmd));
                continue;
            } else if cmd.starts_with(';') {
                // yikes, it's a comment!
                continue;
            }

            if cmd.is_empty() && !dialogue_buffer.is_empty() {
                self.flush_dialogue_buffer(&mut dialogue_buffer, &mut commands);
            } else if !cmd.is_empty() {
                dialogue_buffer.push(cmd.to_owned());
            }
        }

        Ok(commands)
    }
}

impl<R> Parser<R> {
    fn flush_dialogue_buffer(
        &self,
        dialogue_buffer: &mut Vec<String>,
        commands: &mut Vec<Command>,
    ) {
        if dialogue_buffer
            .get(0)
            .and_then(|v| Some(v.starts_with("【")))
            .unwrap_or(false)
        {
            let character_name = dialogue_buffer[0]
                .trim_start_matches("【")
                .trim_end_matches("】")
                .into();
            let text = dialogue_buffer[1..].join("");

            dialogue_buffer.clear();

            commands.push(Command::Dialogue {
                character: Some(character_name),
                text,
            });
        } else {
            let text = dialogue_buffer.join("");

            dialogue_buffer.clear();

            commands.push(Command::Dialogue {
                character: None,
                text,
            });
        }
    }

    fn visit_command(&self, command: &str) -> Command {
        let args: Vec<_> = command.split(',').collect();

        match args[0] {
            "$TITLE" => self.visit_title(&args[1..]),
            "$L_CHR" => self.visit_lchr(&args[1..]),
            "$L_MONT" => self.visit_lmont(&args[1..]),
            "$L_PRIORITY" => self.visit_lpriority(&args[1..]),
            "$DRAW" => self.visit_draw(&args[1..]),
            "$DRAW_EX" => self.visit_draw_ex(&args[1..]),
            "$EX" => self.visit_ex(&args[1..]),
            "$A_CHR" => self.visit_achr(&args[1..]),
            "$L_DELAY" => self.visit_ldelay(&args[1..]),
            "$FACE_AUTO" => self.visit_faceauto(&args[1..]),
            "$FACE_ANIME" => self.visit_faceanime(&args[1..]),
            "$FACE" => self.visit_face(&args[1..]),
            "$MUSIC" => self.visit_music(&args[1..]),
            "$VOICE" => self.visit_voice(&args[1..]),
            "$SE" => self.visit_se(&args[1..]),
            "$MUSIC_FADE" => self.visit_musicfade(&args[1..]),
            "$SE_FADE" => self.visit_sefade(&args[1..]),
            "$WAIT" => self.visit_wait(&args[1..]),
            "$REGMSG" => self.visit_regmsg(&args[1..]),
            "$STR_FLAG" => self.visit_strflag(&args[1..]),
            "$WINDOW" => self.visit_window(&args[1..]),
            "$EMOTION" => self.visit_emotion(&args[1..]),
            "$LABEL" => self.visit_label(&args[1..]),
            "$MOVIE" => self.visit_movie(&args[1..]),
            "$EFECT" => self.visit_effect(&args[1..]),
            "$GLEFECT" => self.visit_gleffect(&args[1..]),
            "$FACET" => Command::Facet,
            _ => {
                panic!("unrecognized command: {}", args[0]);
            }
        }
    }

    fn visit_regmsg(&self, args: &[&str]) -> Command {
        Command::RegMsg {
            unknown: args[0].parse().unwrap(),
        }
    }

    fn visit_effect(&self, args: &[&str]) -> Command {
        Command::Effect {
            unknown: args[0].parse().unwrap(),
            unknown_1: args.get(1).and_then(|v| v.parse().ok()),
        }
    }

    fn visit_gleffect(&self, args: &[&str]) -> Command {
        Command::GlEffect {
            unknown: args.get(0).and_then(|v| v.parse().ok()),
        }
    }

    fn visit_strflag(&self, args: &[&str]) -> Command {
        Command::StrFlag {
            unknown: args[0].parse().unwrap(),
        }
    }

    fn visit_emotion(&self, args: &[&str]) -> Command {
        Command::Emotion {
            layer: args[0].parse().unwrap(),
            filename: args[1].into(),
        }
    }

    fn visit_window(&self, args: &[&str]) -> Command {
        Command::Window {
            unknown: args.get(0).and_then(|v| v.parse().ok()).unwrap_or(0),
        }
    }

    fn visit_label(&self, args: &[&str]) -> Command {
        Command::Label {
            unknown: args[0].parse().unwrap(),
        }
    }

    fn visit_movie(&self, args: &[&str]) -> Command {
        Command::Movie {
            filename: args[0].into(),
            unknown: args[1].parse().unwrap(),
            unknown_1: args[2].parse().unwrap(),
        }
    }

    fn visit_lchr(&self, args: &[&str]) -> Command {
        let layer: i32 = args[0].parse().unwrap();

        if args.len() != 5 {
            return Command::LClear { layer };
        }

        Command::LChr {
            layer,
            filename: args[1].into(),
            x: args[2].parse().unwrap(),
            y: args[3].parse().unwrap(),
            entry: args[4].parse().unwrap(),
        }
    }

    fn visit_lmont(&self, args: &[&str]) -> Command {
        // assert_eq!(args[4], "0");
        assert_eq!(args[5], "m");

        Command::LMont {
            layer: args[0].parse().unwrap(),
            filename: args[1].into(),
            x: args[2].parse().unwrap(),
            y: args[3].parse().unwrap(),
            reserved: args[4].parse().unwrap(),
            entries: args[6..].iter().map(|p| p.parse().unwrap_or(-1)).collect(),
        }
    }

    fn visit_lpriority(&self, args: &[&str]) -> Command {
        if args.is_empty() {
            return Command::LPriorityClear;
        }

        Command::LPriority {
            priority: args.iter().map(|p| p.parse().unwrap()).collect(),
        }
    }

    fn visit_draw(&self, args: &[&str]) -> Command {
        Command::Draw {
            duration: Self::parse_duration(args[0]).unwrap(),
        }
    }

    fn visit_title(&self, args: &[&str]) -> Command {
        Command::Title {
            title: args[0].into(),
        }
    }

    fn visit_draw_ex(&self, args: &[&str]) -> Command {
        match args[0] {
            "0" => Command::DrawExEmpty {
                duration: Self::parse_duration(args[2]).unwrap(),
                unknown: Self::parse_duration(args[3]).unwrap(),
            },
            "2" => Command::DrawEx {
                filename: args[1].into(),
                duration: Self::parse_duration(args[2]).unwrap(),
                reserved_overlay_mode: args[3].parse().unwrap(),
            },
            _ => panic!("unexpected DrawEx pattern"),
        }
    }

    fn visit_ex(&self, args: &[&str]) -> Command {
        if args[0] == "0" {
            return Command::Unknown;
        }

        Command::Ex {
            name: args[0].into(),
            x: args.get(1).unwrap_or(&"0").parse().unwrap(),
            y: args.get(2).unwrap_or(&"0").parse().unwrap(),
        }
    }

    fn visit_achr(&self, args: &[&str]) -> Command {
        Command::AChr {
            id: args[0].parse().unwrap(),
            args: args[1..].iter().copied().map(String::from).collect(),
        }
    }

    fn visit_ldelay(&self, args: &[&str]) -> Command {
        if args[0] == "T" {
            return Command::LDelayAll {
                duration: args
                    .get(1)
                    .copied()
                    .and_then(Self::parse_duration)
                    .unwrap_or(0.0),
            };
        }

        // assert_eq!(args[1], "T");

        Command::LDelay {
            layer: args[0].parse().unwrap(),
            duration: args
                .get(2)
                .copied()
                .and_then(Self::parse_duration)
                .unwrap_or(0.0),
        }
    }

    fn parse_duration(duration: &str) -> Option<f64> {
        duration.parse::<f64>().ok().or_else(|| {
            if duration.ends_with("ms") || duration.ends_with("MS") {
                duration[..duration.len() - 2].parse::<f64>().ok()
            } else if duration.ends_with('s') || duration.ends_with('S') {
                Some(duration[..duration.len() - 1].parse::<f64>().ok()? * 1000.0)
            } else {
                None
            }
        })
    }

    fn visit_faceauto(&self, args: &[&str]) -> Command {
        Command::FaceAuto {
            flag: args[0] != "0",
        }
    }

    fn visit_faceanime(&self, args: &[&str]) -> Command {
        Command::FaceAnime {
            flag: args[0] != "0",
        }
    }

    fn visit_face(&self, args: &[&str]) -> Command {
        if args.is_empty() {
            return Command::FaceClear;
        }

        assert_eq!(args[1], "m");

        Command::Face {
            filename: args[0].into(),
            entries: args[2..]
                .iter()
                .map(|v| v.parse().unwrap_or(-1i32))
                .collect(),
        }
    }

    fn visit_music(&self, args: &[&str]) -> Command {
        Command::Music {
            filename: args[0].into(),
            is_looped: args[1] != "0",
        }
    }

    fn visit_voice(&self, args: &[&str]) -> Command {
        Command::Voice {
            filename: args[0].into(),
        }
    }

    fn visit_se(&self, args: &[&str]) -> Command {
        if args[0].is_empty() {
            println!("warning: invalid SE Syntax; empty");
            return Command::Unknown;
        }

        if args[2].contains('.') {
            println!("warning: invalid SE Syntax");
            return Command::Unknown;
        }

        if args.len() == 3 {
            return Command::SE {
                filename: args[0].into(),
                unknown: args[1].parse().unwrap(),
                channel: 0,
                reserved_delay: args.get(2).copied().and_then(Self::parse_duration),
            };
        }

        Command::SE {
            filename: args[0].into(),
            unknown: args[1].parse().unwrap(),
            channel: args[2].parse().unwrap(),
            reserved_delay: args.get(3).copied().and_then(Self::parse_duration),
        }
    }

    fn visit_musicfade(&self, args: &[&str]) -> Command {
        Command::MusicFade {
            duration: Self::parse_duration(args[0]).unwrap(),
        }
    }

    fn visit_sefade(&self, args: &[&str]) -> Command {
        Command::SEFade {
            duration: Self::parse_duration(args[0]).unwrap(),
            channel: args[1].parse().unwrap(),
        }
    }

    fn visit_wait(&self, args: &[&str]) -> Command {
        Command::Wait {
            duration: Self::parse_duration(args[0]).unwrap(),
        }
    }
}

#[test]
fn parse_rio_script() {
    use encoding_rs::SHIFT_JIS;

    let scenario = include_bytes!("../test/0X_RT_XX.txt");
    let (scenario, _, _) = SHIFT_JIS.decode(scenario);
    let mut parser = Parser::from_raw_bytes(scenario.as_bytes());

    println!("{:#?}", parser.parse());
}

#[test]
fn parse_all_rio_script() {
    for d in std::fs::read_dir("./blob/___t.WAR") {
        for e in d {
            if let Ok(entry) = e {
                if entry.metadata().unwrap().is_dir() {
                    continue;
                }

                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "TXT" {
                        println!("{:?}", path);

                        let mut parser = Parser::open(path).unwrap();
                        println!("{:#?}", parser.parse());
                    }
                }
            }
        }
    }
}
