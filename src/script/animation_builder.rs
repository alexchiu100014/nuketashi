use crate::script::animator::{Animation, AnimationType};
use crate::script::vm::{DrawCall, VmCommand};
use crate::script::state::GameState;

#[derive(Clone)]
enum AnimationSequence {
    Delay(u64),
    Animate(AnimationType),
}

pub struct AnimationBuilder {
    state: GameState,
    anim_per_layer: Vec<Vec<AnimationSequence>>,
}

fn obtain_delay_time(a: &AnimationSequence) -> u64 {
    match a {
        AnimationSequence::Delay(d) => *d,
        _ => 0,
    }
}

impl AnimationBuilder {
    pub fn new(initial_state: GameState) -> Self {
        Self {
            state: initial_state,
            anim_per_layer: vec![vec![]; 30],
        }
    }

    pub fn then(mut self, animation: AnimationType) -> Self {
        self.anim_per_layer[animation.layer() as usize].push(AnimationSequence::Animate(animation));
        self
    }

    pub fn wait(mut self, layer: i32, msecs: u64) -> Self {
        self.anim_per_layer[layer as usize].push(AnimationSequence::Delay(msecs));
        self
    }

    pub fn build(self) -> (Animation, GameState) {
        /* use std::collections::VecDeque;

        let initial_state = self.state;
        let mut anim_per_layer = self.anim_per_layer;
        let mut anim_queue = VecDeque::new();

        while anim_per_layer.iter().any(|a| !a.is_empty()) {
            
        } */

        todo!()
    }
}
