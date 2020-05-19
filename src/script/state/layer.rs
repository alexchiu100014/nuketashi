use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::script::state::DrawCall;
use crate::utils::easing::Easing;

#[derive(Clone, Default)]
pub struct LayerModel {
    // Layer number.
    pub layer_no: i32,
    // S25 filename and entries
    pub filename: Option<PathBuf>,
    pub entries: Vec<i32>,
    // layer property
    pub origin: (f64, f64),
    pub opacity: f32,
    pub blur_radius: (i32, i32),
    // TODO: overlay
    pub overlay: Option<PathBuf>,
    pub overlay_entries: Vec<i32>,
    pub overlay_rate: f32,
    // inner state
    command_queue: VecDeque<LayerCommand>,
    state: LayerState,
    animations: Vec<Animation>,
    finalize_mode: bool,
}

#[derive(Clone, PartialEq)]
pub enum LayerState {
    Idle,
    WaitDraw,
    Timer { wait_until: Instant },
    /* Animate {
        start_time: Instant,
        duration: Duration,
        rate: f64,
        from: AnimationType,
        to: AnimationType,
        easing: Easing,
    }, */
}

#[derive(Clone, PartialEq)]
pub struct Animation {
    pub start_time: Instant,
    pub duration: Duration,
    pub rate: f64,
    pub from: AnimationType,
    pub to: AnimationType,
    pub easing: Easing,
}

impl Default for LayerState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, PartialEq)]
pub enum AnimationType {
    MoveTo(f64, f64),
    MoveBy(f64, f64),
    Opacity(f32),
}

impl AnimationType {
    pub fn interpolate(&self, other: &AnimationType, t: f64) -> AnimationType {
        // clamp to [0, 1]
        let t = 1.0 - t.max(0.0).min(1.0);

        match (self, other) {
            (&AnimationType::MoveTo(x_from, y_from), &AnimationType::MoveTo(x_to, y_to)) => {
                let x = x_to + ((x_from - x_to) * t);
                let y = y_to + ((y_from - y_to) * t);

                AnimationType::MoveTo(x, y)
            }
            (&AnimationType::Opacity(from), &AnimationType::Opacity(to)) => {
                let opacity = to + ((from - to) as f64 * t) as f32;

                AnimationType::Opacity(opacity)
            }
            _ => unreachable!("animation type should match"),
        }
    }
}

#[derive(Clone)]
pub enum LayerCommand {
    LayerClear,
    LayerLoadS25(PathBuf),
    LayerLoadEntries(Vec<i32>),
    LayerDelay(Duration),
    LayerMoveTo(f64, f64),
    LayerOpacity(f32),
    LayerBlur(i32, i32),
    LayerWaitDraw,
    LayerAnimate {
        duration: Duration,
        to: AnimationType,
        easing: Easing,
    },
}

impl LayerModel {
    pub fn new(layer_no: i32) -> Self {
        Self {
            layer_no,
            ..Default::default()
        }
    }

    fn tick(&mut self, now: Instant, command_buffer: &mut Vec<DrawCall>) {
        // animate
        let animations = std::mem::replace(&mut self.animations, vec![]);

        self.animations = animations
            .into_iter()
            .filter_map(|a| {
                if self.finalize_mode || (a.start_time + a.duration) < now {
                    let layer = self.layer_no;

                    match &a.to {
                        &AnimationType::MoveTo(x, y) => {
                            command_buffer.push(DrawCall::LayerMoveTo {
                                layer,
                                origin: (x, y),
                            });
                        }
                        &AnimationType::Opacity(opacity) => {
                            command_buffer.push(DrawCall::LayerOpacity { layer, opacity });
                        }
                        _ => unreachable!("all animation should be transformed to **To format"),
                    }

                    self.state = LayerState::Idle;
                    return None;
                } else if now < a.start_time {
                    return Some(a);
                }

                let delta_time = (now - a.start_time).as_secs_f64();
                let t = delta_time * a.rate;
                let res = a.from.interpolate(&a.to, a.easing.apply(t));
                let layer = self.layer_no;

                match res {
                    AnimationType::MoveTo(x, y) => {
                        command_buffer.push(DrawCall::LayerMoveTo {
                            layer,
                            origin: (x, y),
                        });
                    }
                    AnimationType::Opacity(opacity) => {
                        command_buffer.push(DrawCall::LayerOpacity { layer, opacity });
                    }
                    _ => unreachable!("all animation should be transformed to **To format"),
                }

                Some(a)
            })
            .collect();

        // state
        match &self.state {
            LayerState::Idle => {
                // do nothing
            }
            LayerState::WaitDraw => {
                if self.finalize_mode {
                    self.state = LayerState::Idle;
                }
            }
            LayerState::Timer { wait_until } => {
                // foce finalize
                if self.finalize_mode {
                    self.state = LayerState::Idle;
                    return;
                }

                // check timer
                if now < *wait_until {
                    return;
                }

                self.state = LayerState::Idle;
            }
        }
    }

    pub fn poll(&mut self, now: Instant, command_buffer: &mut Vec<DrawCall>) {
        // set to idle (workaround for slow texture loading)
        if self.state == LayerState::WaitDraw {
            self.state = LayerState::Idle;
        }

        loop {
            // generate state
            self.update(now, command_buffer);

            // proceed current event
            self.tick(now, command_buffer);

            if (!self.finalize_mode && LayerState::Idle != self.state)
                || self.command_queue.is_empty()
            {
                break;
            }
        }

        self.finalize_mode = false;
    }

    pub fn send(&mut self, command: LayerCommand) {
        self.command_queue.push_back(command);
    }

    pub fn finalize(&mut self) {
        self.finalize_mode = true;
    }

    pub fn update(&mut self, now: Instant, command_buffer: &mut Vec<DrawCall>) {
        let layer = self.layer_no;

        // if the layer is not ready, ignore
        // and let the poller finish all the event
        if LayerState::Idle != self.state {
            return;
        }

        // process a command
        match self.command_queue.pop_front() {
            Some(LayerCommand::LayerWaitDraw) => {
                self.state = LayerState::WaitDraw;
            }
            Some(LayerCommand::LayerClear) => {
                self.filename = None;
                self.entries = vec![];
                command_buffer.push(DrawCall::LayerClear { layer });
            }
            Some(LayerCommand::LayerLoadS25(filename)) => {
                self.filename = Some(filename.clone());
                command_buffer.push(DrawCall::LayerLoadS25 {
                    layer,
                    path: filename,
                });
            }
            Some(LayerCommand::LayerLoadEntries(entries)) => {
                self.entries = entries.clone();
                command_buffer.push(DrawCall::LayerSetCharacter {
                    layer,
                    pict_layers: entries,
                });
            }
            Some(LayerCommand::LayerMoveTo(x, y)) => {
                self.origin = (x, y);

                command_buffer.push(DrawCall::LayerMoveTo {
                    layer,
                    origin: (x, y),
                });
            }
            Some(LayerCommand::LayerOpacity(opacity)) => {
                self.opacity = opacity;
                command_buffer.push(DrawCall::LayerOpacity { layer, opacity });
            }
            Some(LayerCommand::LayerBlur(x, y)) => {
                self.blur_radius = (x, y);
                command_buffer.push(DrawCall::LayerBlur {
                    layer,
                    radius: (x, y),
                });
            }
            Some(LayerCommand::LayerDelay(t)) => {
                if self.finalize_mode {
                    self.state = LayerState::Idle;
                } else {
                    self.state = LayerState::Timer {
                        wait_until: now + t,
                    };
                }
            }
            Some(LayerCommand::LayerAnimate {
                duration,
                to,
                easing,
            }) => {
                let (initial_state, to) = match &to {
                    &AnimationType::MoveTo(x, y) => {
                        let (x_from, y_from) = self.origin;
                        self.origin = (x, y);
                        (AnimationType::MoveTo(x_from, y_from), to)
                    }
                    &AnimationType::MoveBy(x, y) => {
                        let (x_from, y_from) = self.origin;
                        self.origin = (self.origin.0 + x, self.origin.1 + y);

                        // translate MoveBy to MoveTo
                        (
                            AnimationType::MoveTo(x_from, y_from),
                            AnimationType::MoveTo(self.origin.0, self.origin.1),
                        )
                    }
                    &AnimationType::Opacity(opacity) => {
                        let opacity_from = self.opacity;
                        self.opacity = opacity;
                        (AnimationType::Opacity(opacity_from), to)
                    }
                };

                self.animations.push(Animation {
                    start_time: now,
                    duration,
                    rate: 1.0 / duration.as_secs_f64(),
                    from: initial_state,
                    to,
                    easing,
                });
            }
            _ => {
                // ignore
            }
        }
    }
}
