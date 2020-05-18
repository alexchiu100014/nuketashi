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
    pub origin: (i32, i32),
    pub opacity: f32,
    pub blur_radius: (i32, i32),
    // TODO: overlay
    pub overlay: Option<PathBuf>,
    pub overlay_entries: Vec<i32>,
    pub overlay_rate: f32,
    // inner state
    command_queue: VecDeque<LayerCommand>,
    state: LayerState,
    finalize_mode: bool,
}

#[derive(Clone, PartialEq)]
pub enum LayerState {
    Idle,
    WaitDraw,
    Timer {
        wait_until: Instant,
    },
    Animate {
        start_time: Instant,
        duration: Duration,
        rate: f64,
        from: AnimationType,
        to: AnimationType,
        easing: Easing,
    },
}

impl Default for LayerState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, PartialEq)]
pub enum AnimationType {
    MoveTo(i32, i32),
    MoveBy(i32, i32),
    Opacity(f32),
}

impl AnimationType {
    pub fn interpolate(&self, other: &AnimationType, t: f64) -> AnimationType {
        // clamp to [0, 1]
        let t = 1.0 - t.max(0.0).min(1.0);

        match (self, other) {
            (&AnimationType::MoveTo(x_from, y_from), &AnimationType::MoveTo(x_to, y_to)) => {
                let x = x_to + ((x_from - x_to) as f64 * t) as i32;
                let y = y_to + ((y_from - y_to) as f64 * t) as i32;

                AnimationType::MoveTo(x, y)
            }
            (&AnimationType::Opacity(from), &AnimationType::Opacity(to)) => {
                let opacity = from + ((to - from) as f64 * t) as f32;

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
    LayerMoveTo(i32, i32),
    LayerOpacity(f32),
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
            LayerState::Animate {
                start_time,
                duration,
                rate,
                from,
                to,
                easing,
            } => {
                if self.finalize_mode || (*start_time + *duration) < now {
                    let layer = self.layer_no;

                    match to {
                        &AnimationType::MoveTo(x, y) => {
                            command_buffer.push(DrawCall::LayerMoveTo {
                                layer,
                                origin: (x, y),
                            });
                        }
                        &AnimationType::Opacity(_opacity) => {
                            /* command_buffer.push(DrawCall::LayerSetOpacity {
                                layer,
                                opacity
                            }); */
                        }
                        _ => unreachable!("all animation should be transformed to **To format"),
                    }

                    self.state = LayerState::Idle;
                    return;
                } else if now < *start_time {
                    return;
                }

                let delta_time = (now - *start_time).as_secs_f64();
                let t = delta_time * *rate;
                let res = from.interpolate(&to, easing.apply(t));
                let layer = self.layer_no;

                match res {
                    AnimationType::MoveTo(x, y) => {
                        command_buffer.push(DrawCall::LayerMoveTo {
                            layer,
                            origin: (x, y),
                        });
                    }
                    AnimationType::Opacity(_opacity) => {
                        /* command_buffer.push(DrawCall::LayerSetOpacity {
                            layer,
                            opacity
                        }); */
                    }
                    _ => unreachable!("all animation should be transformed to **To format"),
                }
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
                // command_buffer.push(DrawCall::LayerSetOpacity { layer, opacity });
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

                self.state = LayerState::Animate {
                    start_time: now,
                    duration,
                    rate: 1.0 / duration.as_secs_f64(),
                    from: initial_state,
                    to,
                    easing,
                };
            }
            _ => {
                // ignore
            }
        }
    }
}
