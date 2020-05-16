use crate::script::animator::{Animation, AnimationType};
use crate::script::state::GameState;
use crate::script::vm::DrawCall;

use std::collections::VecDeque;

#[derive(Clone)]
enum AnimationSequence {
    Delay(u64),
    Animate(AnimationType),
}

pub struct AnimationBuilder {
    state: GameState,
    anim_per_layer: Vec<VecDeque<AnimationSequence>>,
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
            anim_per_layer: vec![Default::default(); 30],
        }
    }

    pub fn then(mut self, animation: AnimationType) -> Self {
        self.anim_per_layer[animation.layer() as usize]
            .push_back(AnimationSequence::Animate(animation));
        self
    }

    pub fn wait(mut self, layer: i32, msecs: u64) -> Self {
        self.anim_per_layer[layer as usize].push_back(AnimationSequence::Delay(msecs));
        self
    }

    pub fn build(self) -> Animation {
        let mut state = self.state;
        let mut anim_per_layer = self.anim_per_layer;
        let mut anim_queue: VecDeque<(Option<AnimationType>, _)> = VecDeque::new();

        while anim_per_layer.iter().any(|a| !a.is_empty()) {
            let (i, msec) = anim_per_layer
                .iter()
                .enumerate()
                .fold(None, |p, (i, v)| match (p, v.get(0)) {
                    (None, Some(v)) => Some((i, obtain_delay_time(v))),
                    (Some((_, p)), Some(v)) if obtain_delay_time(v) < p => {
                        Some((i, obtain_delay_time(v)))
                    }
                    _ => p,
                })
                .unwrap();

            if msec == 0 {
                let anim = anim_per_layer[i].pop_front().unwrap();

                match &anim {
                    &AnimationSequence::Animate(AnimationType::LayerPosition {
                        layer,
                        position: origin,
                    }) => {
                        anim_queue.push_back((
                            Some(AnimationType::LayerPosition {
                                layer,
                                position: state.layers[layer as usize].origin,
                            }),
                            anim,
                        ));

                        state.send_draw_call(&DrawCall::LayerMoveTo { layer, origin });
                    }
                    AnimationSequence::Animate(AnimationType::LayerOpacity { .. }) => {
                        // state.send_draw_call(&DrawCall::LayerOpacity {layer , opacity});
                        anim_queue.push_back((None, anim));
                    }
                    _ => {}
                }
            } else {
                for a in &mut anim_per_layer {
                    if let Some(a) = a.get_mut(0) {
                        if let AnimationSequence::Delay(d) = a {
                            *d -= msec;
                        }
                    }
                }

                anim_queue.push_back((None, AnimationSequence::Delay(msec)));
            }
        }
        
        todo!()
    }
}
