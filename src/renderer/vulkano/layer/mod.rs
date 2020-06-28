pub mod layer_texture;
pub mod pict_layer;

// use layer_texture::LayerTexture;
use pict_layer::{PictLayer, Vertex};

use vulkano::buffer::ImmutableBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::{vertex::VertexSource, GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sync::GpuFuture;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::constants::LRU_CACHE_CAPACITY;
use crate::format::s25::S25Archive;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum OverlayMode {
    Disabled,
    Normal,
    Reverse,
}

impl Default for OverlayMode {
    fn default() -> Self {
        Self::Disabled
    }
}

use lru::LruCache;
use std::sync::RwLock;

pub type PictLayerEntry = Arc<RwLock<PictLayer>>;

pub struct LayerRenderer {
    // .s25 thingy
    pub s25: Option<S25Archive>,
    pub filename: Option<String>,
    // pict-layer entries and cache
    pub entries: Vec<PictLayerEntry>,
    pub cache: LruCache<(String, i32), PictLayerEntry>,
    // property
    pub offset: (i32, i32),
    pub opacity: f32,
    pub blur: Option<(i32, i32)>,
    // for optimization
    update_flag: bool,
    queued_load: Option<(String, Vec<i32>)>,
    queued_prefetch: Option<(String, Vec<i32>)>,
    //
    format: Format,
}

impl LayerRenderer {
    pub fn new(format: Format) -> Self {
        Self {
            s25: None,
            filename: None,
            entries: vec![],
            cache: LruCache::new(LRU_CACHE_CAPACITY),
            opacity: 1.0,
            update_flag: false,
            blur: None,
            offset: (0, 0),
            queued_load: None,
            queued_prefetch: None,
            format,
        }
    }

    fn load_entry<Mv, L, Rp>(
        &mut self,
        entry: i32,
        queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) -> Option<PictLayerEntry>
    where
        L: PipelineLayoutAbstract,
        Rp: RenderPassAbstract,
    {
        if entry < 0 {
            return None;
        }

        if let Some(cached) = self.cache.get(&(self.filename.clone()?, entry)) {
            return Some(cached.clone());
        }

        log::warn!(
            "image not cached ({:?}@{}); might cause frame drops",
            entry,
            self.filename.as_deref().unwrap()
        );

        let image = self.s25.as_mut()?.load_image(entry as usize).ok()?;
        let mut layer = PictLayer::empty();
        layer.load_gpu(image, queue, pipeline, self.format);

        let layer = Arc::new(RwLock::new(layer));

        self.cache
            .put((self.filename.clone()?, entry), layer.clone());

        Some(layer)
    }

    fn open_s25(filename: &str) -> Option<S25Archive> {
        S25Archive::open(Self::lookup(filename.split('\\').last().unwrap())).ok()
    }

    fn prefetch_entry<Mv, L, Rp>(
        &mut self,
        filename: &str,
        entry: i32,
        queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) -> Option<()>
    where
        L: PipelineLayoutAbstract,
        Rp: RenderPassAbstract,
    {
        use std::time::Instant;

        if entry < 0 || self.cache.get(&(filename.into(), entry)).is_some() {
            log::debug!("already cached: {}@{}", entry, filename);
            return None;
        }

        let now = Instant::now();

        let image = if self
            .filename
            .as_ref()
            .map(|s| s == filename)
            .unwrap_or_default()
        {
            self.s25.as_mut().unwrap().load_image(entry as usize)
        } else {
            let mut s25 = Self::open_s25(filename)?;
            s25.load_image(entry as usize)
        }
        .ok()?;

        log::info!("decode took {} ms", (Instant::now() - now).as_millis());

        let mut layer = PictLayer::empty();
        layer.load_gpu(image, queue, pipeline, self.format);

        let layer = Arc::new(RwLock::new(layer));

        self.cache.put((filename.into(), entry), layer.clone());

        log::debug!("successfully cached: {}@{}", entry, filename);

        Some(())
    }

    pub fn load<Mv, L, Rp>(
        &mut self,
        filename: &str,
        entries: &[i32],
        queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) where
        L: PipelineLayoutAbstract,
        Rp: RenderPassAbstract,
    {
        self.filename = Some(filename.into());
        self.s25 = Self::open_s25(filename);

        self.entries = entries
            .iter()
            .copied()
            .filter_map(|e| self.load_entry(e, queue.clone(), pipeline.clone()))
            .collect();

        self.update_flag = true;
    }

    pub fn unload(&mut self) {
        self.filename = None;
        self.s25 = None;
        self.entries = vec![];
        self.update_flag = true;
    }

    pub fn prefetch<Mv, L, Rp>(
        &mut self,
        filename: &str,
        entries: &[i32],
        queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) where
        L: PipelineLayoutAbstract,
        Rp: RenderPassAbstract,
    {
        log::debug!("prefetch: {:?}@{}", entries, filename);

        for &e in entries {
            self.prefetch_entry(filename, e, queue.clone(), pipeline.clone());
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.offset = (x, y);
    }

    pub fn set_blur_rate(&mut self, rx: i32, ry: i32) {
        self.blur = Some((rx, ry));
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    pub fn update<Mv, L, Rp>(
        &mut self,
        queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) where
        L: PipelineLayoutAbstract,
        Rp: RenderPassAbstract,
    {
        if !self.update_flag {
            return;
        }

        if let Some((filename, entries)) = self.queued_load.take() {
            self.load(&filename, &entries, queue.clone(), pipeline.clone());
        }

        if let Some((filename, entries)) = self.queued_prefetch.take() {
            self.prefetch(&filename, &entries, queue.clone(), pipeline.clone());
        }

        self.update_flag = false;
    }

    pub fn draw<P>(
        &self,
        builder: &mut AutoCommandBufferBuilder,
        pipeline: P,
        dyn_state: &DynamicState,
    ) where
        P: GraphicsPipelineAbstract
            + VertexSource<Arc<ImmutableBuffer<[Vertex]>>>
            + Send
            + Sync
            + 'static
            + Clone,
    {
        if self.entries.is_empty() {
            return;
        }

        // let all the pict-layers draw
        for layer in &self.entries {
            let layer = layer.read().unwrap();

            assert!(layer.is_cached(), "layer not cached");

            layer.draw(
                builder,
                pipeline.clone(),
                dyn_state,
                (self.offset.0 as f64, self.offset.1 as f64),
                self.opacity,
                self.blur.unwrap_or_default(),
            );
        }
    }

    pub fn take_future(&mut self, device: Arc<Device>) -> Box<dyn GpuFuture> {
        let mut future = Box::new(vulkano::sync::now(device)) as Box<dyn GpuFuture>;

        for layer in &self.entries {
            let mut layer = layer.write().unwrap();
            if let Some(f) = layer.take_future() {
                future = Box::new(future.join(f));
            }
        }

        future
    }
}

use crate::renderer::vulkano::{VulkanoBackend, VulkanoRenderingContext, VulkanoRenderingTarget};
use crate::renderer::Renderer;

pub struct LayerRenderingContext {
    pub render_pass: Arc<dyn RenderPassAbstract + Sync + Send>,
    pub pipeline: Arc<
        GraphicsPipeline<
            SingleBufferDefinition<Vertex>,
            Box<dyn PipelineLayoutAbstract + Send + Sync>,
            Arc<dyn RenderPassAbstract + Sync + Send>,
        >,
    >,
}

impl VulkanoRenderingContext for LayerRenderingContext {
    fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Sync + Send> {
        &self.render_pass
    }
}

impl<T> Renderer<VulkanoBackend, T> for LayerRenderer
where
    T: VulkanoRenderingTarget,
{
    type Context = LayerRenderingContext;

    fn render(&mut self, target: &mut T, ctx: &Self::Context) {
        if self.entries.is_empty() {
            return;
        }

        let state = target.dynamic_state().clone();

        self.draw(target.command_buffer(), ctx.pipeline.clone(), &state);
    }
}

// command receiver

use crate::script::mil::command::LayerCommand;

impl LayerRenderer {
    pub fn send(&mut self, command: LayerCommand) {
        match command {
            LayerCommand::Load(filename, entries) => {
                let entries: Vec<_> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| if v == -1 { -1 } else { v + (i as i32) * 100 })
                    .collect();

                log::debug!("load: {}, {:?}", filename, entries);

                self.queued_load = Some((filename, entries));
                self.update_flag = true;
            }
            LayerCommand::Unload => {
                log::debug!("unload");
                self.unload();
            }
            LayerCommand::Prefetch(filename, entries) => {
                let entries: Vec<_> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| if v == -1 { -1 } else { v + (i as i32) * 100 })
                    .collect();

                log::debug!("prefetch: {}, {:?}", filename, entries);

                self.queued_prefetch = Some((filename, entries));
                self.update_flag = true;
            }
            LayerCommand::SetPosition(x, y) => {
                log::debug!("position: {}, {}", x, y);
                self.set_position(x as i32, y as i32);
            }
            LayerCommand::SetOpacity(opacity) => {
                log::debug!("opacity: {}", opacity);
                self.set_opacity(opacity as f32);
            }
            LayerCommand::SetBlurRate(rx, ry) => {
                log::debug!("blur rate: ({}, {})", rx, ry);
                self.set_blur_rate(rx, ry);
            }
            LayerCommand::LoadOverlay(path, entry, mode) => {
                log::debug!("overlay: {}, {}, {}", path, entry, mode);
                log::error!("overlay not supported");
            } // filename, entry, overlay mode
            LayerCommand::UnloadOverlay => {
                log::debug!("overlay unload");
                log::error!("overlay unload not supported");
            }
            LayerCommand::SetOverlayRate(rate) => {
                log::debug!("overlay rate: {}", rate);
                log::error!("overlay rate not supported");
            }
            LayerCommand::LoadAnimationGraph(graph) => {
                log::debug!("anim graph: {:?}", graph);
                log::error!("anim graph not supported");
            }
            LayerCommand::WaitUntilAnimationIsDone => {
                log::debug!("wait until animation is done");
                log::error!("anim not supported");
            }
            LayerCommand::FinalizeAnimation => {
                log::debug!("finalize anim");
                log::error!("anim not supported");
            }
            LayerCommand::LayerDelay(v) => {
                log::debug!("layer delay: {}", v);
                log::error!("anim not supported");
            }
        }
    }
}

// directory lookup

impl LayerRenderer {
    fn lookup_into(filename: &str, dir: &Path) -> Option<PathBuf> {
        for d in std::fs::read_dir(dir) {
            for e in d {
                if let Ok(entry) = e {
                    if entry.metadata().unwrap().is_dir() {
                        if let Some(r) = Self::lookup_into(filename, &entry.path()) {
                            return Some(r);
                        }
                    }

                    let path = entry.path();
                    let entry_name = path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_ascii_uppercase();
                    let entry_stem = path
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_ascii_uppercase();

                    if entry_stem.ends_with("(1)")
                        && filename.starts_with(entry_stem.trim_end_matches("(1)"))
                    {
                        return Some(entry.path().into());
                    } else if entry_name == filename {
                        return Some(entry.path().into());
                    }
                }
            }
        }

        None
    }

    fn lookup(filename: &str) -> PathBuf {
        // TODO
        Self::lookup_into(&filename.to_ascii_uppercase(), "./blob/".as_ref()).unwrap()
    }
}
