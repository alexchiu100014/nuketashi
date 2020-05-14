//
use super::vm::{DrawCall, VmCommand};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub enum AnimationType {
    LayerPosition { layer: i32, position: (i32, i32) },
    LayerOpacity { layer: i32, opacity: f32 },
    Nothing,
}

#[derive(Clone)]
pub struct Animation {
    pub anim_from: Option<AnimationType>,
    pub anim_to: AnimationType,
    pub repeated: bool,
    pub then: Vec<VmCommand>,
    pub finally: Vec<VmCommand>,
    start_at: Option<Instant>,
    end_at: Option<Instant>,
    duration: Duration,
    inner: f64,
    is_done: bool,
    time_rate: f64,
}

impl Animation {
    pub fn new(
        anim_from: Option<AnimationType>,
        anim_to: AnimationType,
        repeated: bool,
        duration: Duration,
    ) -> Self {
        Self {
            inner: 0.0,
            is_done: false,
            time_rate: 1.0 / duration.as_secs_f64(),
            then: vec![],
            finally: vec![],
            anim_from,
            anim_to,
            repeated,
            duration,
            start_at: None,
            end_at: None,
        }
    }

    pub fn start(&mut self) {
        assert!(self.anim_from.is_some());

        let now = Instant::now();
        self.start_at = Some(now);
        self.end_at = Some(now + self.duration);
    }

    pub fn stop(&mut self) {
        self.start_at = None;
        self.end_at = None;
        self.inner = 1.0;
        self.is_done = true;
    }

    pub fn reset(&mut self) {
        self.start_at = None;
        self.end_at = None;
        self.inner = 0.0;
        self.is_done = false;
    }

    pub fn tick(&mut self, now: Instant) {
        if self.is_done || self.start_at.map(|t| now < t).unwrap_or(false) {
            return;
        } else if self.end_at.unwrap() < now {
            self.is_done = true;
            self.inner = 1.0;
            return;
        }

        self.inner = (now - self.start_at.unwrap()).as_secs_f64() * self.time_rate;
    }

    pub fn poll(&mut self) -> Vec<VmCommand> {
        if self.is_done {
            // animation done successfully
            if self.end_at.is_some() {
                if self.repeated {
                    let mut s = self.clone();
                    s.reset();
                    self.then.push(VmCommand::Animate(s));
                }

                self.then.append(&mut self.finally);
                std::mem::swap(&mut self.finally, &mut self.then);
            }

            std::mem::replace(&mut self.finally, vec![])
        } else {
            self.interpolate()
                .into_iter()
                .map(|d| VmCommand::Draw(d))
                .collect()
        }
    }

    pub fn interpolate(&mut self) -> Option<DrawCall> {
        match (self.anim_from, self.anim_to) {
            (None, _) => {
                log::error!("animation should have previous state.");
            }
            (
                Some(AnimationType::LayerPosition {
                    layer,
                    position: (x_from, y_from),
                }),
                AnimationType::LayerPosition {
                    layer: layer_,
                    position: (x_to, y_to),
                },
            ) if layer == layer_ => {
                let x = x_to - ((x_to - x_from) as f64 * (1.0 - self.inner)) as i32;
                let y = y_to - ((y_to - y_from) as f64 * (1.0 - self.inner)) as i32;

                return Some(DrawCall::LayerMoveTo {
                    layer,
                    origin: (x, y),
                });
            }
            (
                Some(AnimationType::LayerOpacity {
                    layer,
                    opacity: opacity_from,
                }),
                AnimationType::LayerOpacity {
                    layer: layer_,
                    opacity: opacity_to,
                },
            ) if layer == layer_ => {
                let _opacity =
                    opacity_to - ((opacity_to - opacity_from) as f64 * (1.0 - self.inner)) as f32;

                /* return Some(DrawCall::LayerOpacity {
                    layer,
                    opacity,
                }); */
            }
            (Some(AnimationType::Nothing), AnimationType::Nothing) => {
                return None;
            }
            _ => {
                log::warn!("unrecognized animation");
            }
        }

        None
    }
}

#[derive(Default)]
pub struct Animator {
    pub animations: Vec<Animation>,
}

impl Animator {
    pub fn tick(&mut self) {
        let now = Instant::now();

        for anim in &mut self.animations {
            anim.tick(now);
        }
    }

    pub fn poll(&mut self) -> Vec<VmCommand> {
        let mut v = vec![];
        for anim in &mut self.animations {
            v.append(&mut anim.poll());
        }
        v
    }
}
