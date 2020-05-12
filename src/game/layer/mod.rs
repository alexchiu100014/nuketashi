use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::ImmutableImage;
use vulkano::sync::NowFuture;

use std::sync::Arc;

use crate::format::s25::{S25Archive, S25Image};
use crate::game::texture_loader;
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture,
};

pub type Texture = Arc<ImmutableImage<Format>>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum OverlayMode {
    Disabled,
    Normal,
    Reverse,
}

#[derive(Default)]
pub struct PictLayer {
    pub entry_no: i32,
    pub texture: Option<Texture>,
    pub future: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
    pub offset: (i32, i32),
    pub size: (i32, i32),
    pub set: Option<Box<dyn DescriptorSet>>,
}

impl PictLayer {
    pub fn empty() -> Self {
        Self {
            entry_no: -1,
            ..Self::default()
        }
    }

    // load pict-layer information onto GPU
    pub fn load_gpu(&mut self, image: S25Image, load_queue: Arc<Queue>) {
        // load image to GPU

        let offset = (image.metadata.offset_x, image.metadata.offset_y);
        let size = (image.metadata.width, image.metadata.height);
        let (t, f) = texture_loader::load_s25_image(image, load_queue);

        self.texture = Some(t);
        self.future = Some(f);
        self.offset = offset;
        self.size = size;

        // load other information to GPU
    }

    pub fn draw(&self, builder: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
        builder
    }
}

pub struct Layer {
    // S25 archive that corresponds to the layer
    pub s25_archive: Option<S25Archive>,
    // parameters for pict layers
    pub pict_layers: Vec<PictLayer>,
    pub overlay: Option<Texture>,
    pub overlay_future: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
    // value sent to the shader to render
    pub overlay_mode: OverlayMode,
    pub overlay_rate: f32, // [0, 1]
    pub position: (i32, i32),
    pub opacity: f32, // [0, 1]
}

impl Layer {
    pub fn load_s25(&mut self, s25: S25Archive) {
        // clear layer
        self.clear_layers();

        // replace s25 file
        self.s25_archive = Some(s25);
    }

    pub fn load_pict_layers(&mut self, pict_layers: &[i32], load_queue: Arc<Queue>) {
        // match the length of pict layers
        self.pict_layers
            .resize_with(pict_layers.len(), PictLayer::empty);

        if let Some(arc) = &mut self.s25_archive {
            for (i, &entry) in pict_layers.iter().enumerate() {
                let pict_layer = &mut self.pict_layers[i];
                // don't reload if the image is the same
                if pict_layer.entry_no == entry {
                    continue;
                }

                pict_layer.entry_no = entry;

                // clear the pict-layer is -1 is given
                if entry == -1 {
                    pict_layer.texture = None;
                    pict_layer.future = None;
                    continue;
                }

                let img = arc
                    .load_image(entry as usize)
                    .expect("failed to load the image entry");

                // load pict-layer information to GPU
                pict_layer.load_gpu(img, load_queue.clone());
            }
        }
    }

    pub fn load_overlay(&mut self, overlay: S25Image, load_queue: Arc<Queue>) {
        let (t, f) = texture_loader::load_s25_image(overlay, load_queue);
        self.overlay = Some(t);
        self.overlay_future = Some(f);
    }

    pub fn clear_layers(&mut self) {
        self.pict_layers.clear();

        self.overlay.take();
        self.overlay_future.take();

        // make opacity and overlay_rate zero
        self.opacity = 0.0;
        self.overlay_rate = 0.0;

        // disable overlay mode
        self.overlay_mode = OverlayMode::Disabled;
    }

    pub fn draw(&self, builder: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
        let mut builder = builder;

        // let all the pict-layers draw
        for layer in &self.pict_layers {
            builder = layer.draw(builder);
        }

        // apply overlay

        builder
    }
}
