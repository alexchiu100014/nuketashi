use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::script::state::DrawCall;

#[derive(Clone, Default)]
pub struct LayerModel {
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
}

#[derive(Clone, PartialEq)]
pub enum LayerState {
    Idle,
    Timer {
        wait_until: Instant,
    },
    Animate {
        start_time: Instant,
        duration: Duration,
        rate: f64,
        from: AnimationType,
        to: AnimationType,
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
    Opacity(f32),
}

impl AnimationType {
    pub fn interpolate(&self, other: &AnimationType, t: f64) -> AnimationType {
        // clamp to [0, 1]
        let t = t.max(0.0).min(1.0);

        match (self, other) {
            (&AnimationType::MoveTo(x_from, y_from), &AnimationType::MoveTo(x_to, y_to)) => {
                let x = x_from + ((x_to - x_from) as f64 * t) as i32;
                let y = y_from + ((y_to - y_from) as f64 * t) as i32;

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
    LayerLoadS25(PathBuf),
    LayerLoadEntries(Vec<i32>),
    LayerDelay(Duration),
    LayerMoveTo(i32, i32),
    LayerOpacity(f32),
    LayerAnimate {
        duration: Duration,
        to: AnimationType,
    },
}

impl LayerModel {
    fn tick(&mut self, now: Instant, command_buffer: &Vec<DrawCall>) {
        match &self.state {
            LayerState::Idle => {
                // do nothing
            }
            LayerState::Timer { wait_until } => {
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
            } => {
                if now < *start_time {
                    return;
                } else if (*start_time + *duration) < now {
                    self.state = LayerState::Idle;
                    return;
                }

                let delta_time = (now - *start_time).as_secs_f64();
                let res = from.interpolate(&to, delta_time * *rate);
            }
        }
    }

    pub fn poll(&mut self, now: Instant, command_buffer: &Vec<DrawCall>) {
        loop {
            // proceed current event
            self.tick(now, command_buffer);

            // generate state
            self.update(now, command_buffer);

            if LayerState::Idle != self.state {
                break;
            }
        }
    }

    pub fn update(&mut self, now: Instant, command_buffer: &Vec<DrawCall>) {
        if LayerState::Idle != self.state {
            return;
        }

        // proceed command
        match self.command_queue.pop_front() {
            Some(LayerCommand::LayerLoadS25(filename)) => {}
            Some(LayerCommand::LayerLoadEntries(entries)) => {}
            Some(LayerCommand::LayerMoveTo(x, y)) => {}
            Some(LayerCommand::LayerDelay(t)) => {}
            Some(LayerCommand::LayerOpacity(opacity)) => {}
            Some(LayerCommand::LayerAnimate { duration, to }) => {}
            _ => {
                // ignore
            }
        }
    }
}
