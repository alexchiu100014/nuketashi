use super::{LayerCommand, LayerModel};
use crate::utils::easing::Easing;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Animation {
    pub start_time: Instant,
    pub duration: Duration,
    pub rate: f64,
    pub from: AnimationType,
    pub to: AnimationType,
    pub easing: Easing,
    pub then: Vec<LayerCommand>,
}

#[derive(Clone, PartialEq)]
pub enum AnimationType {
    MoveTo(f64, f64),
    MoveBy(f64, f64),
    Opacity(f32),
}

struct AnimationEntry {
    duration: Duration,
    to: AnimationType,
    easing: Easing,
}

pub struct AnimationBuilder {
    animations: VecDeque<AnimationEntry>,
}

impl AnimationBuilder {
    pub fn new() -> Self {
        Self {
            animations: Default::default(),
        }
    }

    pub fn next(mut self, duration: Duration, to: AnimationType, easing: Easing) -> Self {
        self.animations.push_back(AnimationEntry {
            duration,
            to,
            easing,
        });

        self
    }

    pub fn build(self, layer: &mut LayerModel) {
        let mut then = vec![];
        let mut a = self.animations;

        while let Some(AnimationEntry {
            duration,
            to,
            easing,
            ..
        }) = a.pop_back()
        {
            let cmd = LayerCommand::LayerAnimate {
                duration,
                to,
                easing,
                then,
            };

            then = vec![];
            then.push(cmd);
        }

        if let Some(cmd) = then.pop() {
            layer.send(cmd);
        }
    }
}
