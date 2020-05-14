//
use super::vm::{DrawCall, VmCommand};
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum AnimationType {
    LayerPosition { layer: i32, position: (i32, i32) },
    LayerOpacity { layer: i32, opacity: f32 },
}

#[derive(Clone)]
pub struct Animation {
    pub anim_from: Option<AnimationType>,
    pub anim_to: AnimationType,
    pub repeated: bool,
    pub start_at: Instant,
    pub end_at: Instant,
    pub then: Vec<VmCommand>,
    pub finally: Vec<VmCommand>,
    inner: f64,
    is_done: bool,
    time_rate: f64,
}

impl Animation {
    pub fn new(
        anim_from: Option<AnimationType>,
        anim_to: AnimationType,
        repeated: bool,
        start_at: Instant,
        end_at: Instant,
    ) -> Self {
        Self {
            inner: 0.0,
            is_done: end_at < start_at,
            time_rate: 1.0 / (end_at - start_at).as_secs_f64(),
            then: vec![],
            finally: vec![],
            anim_from,
            anim_to,
            repeated,
            start_at,
            end_at,
        }
    }

    pub fn tick(&mut self, now: Instant) {
        if self.is_done {
            return;
        } else if self.end_at < now {
            self.is_done = true;
            self.inner = 1.0;
            return;
        }

        self.inner = (now - self.start_at).as_secs_f64() * self.time_rate;
    }

    /* pub fn poll(&mut self) -> Vec<VmCommand> {
    if self.is_done {

    } else {
        vec!
    }
    } */

    pub fn interpolate(&mut self) -> DrawCall {
        match (self.anim_from, self.anim_to) {
            (None, _) => {
                log::error!("animation should have previous state.");
            }
            _ => {
                log::warn!("unrecognized animation");
            }
        }

        todo!()
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
}
