use crate::format::s25::S25Archive;
use crate::renderer::Renderer;

use crate::renderer::cpu::image::Image;
use crate::renderer::cpu::{CpuBackend, CpuImageBuffer};

use std::path::Path;
use std::sync::Arc;

use lru::LruCache;

use crate::constants::{GAME_WINDOW_HEIGHT, GAME_WINDOW_WIDTH, LRU_CACHE_CAPACITY};

#[derive(Clone)]
pub struct PictLayer {
    pub image: Image,
    pub offset: (i32, i32),
}

pub struct LayerRenderer {
    pub s25: Option<S25Archive>,
    pub filename: Option<String>,
    pub entries: Vec<Arc<PictLayer>>,
    pub cache: LruCache<(String, i32), Arc<PictLayer>>,
    pub framebuffer: Image,
    pub opacity: f32,
}

impl LayerRenderer {
    pub fn new() -> Self {
        Self {
            s25: None,
            filename: None,
            entries: vec![],
            cache: LruCache::new(LRU_CACHE_CAPACITY),
            framebuffer: Image::new(GAME_WINDOW_WIDTH as usize, GAME_WINDOW_HEIGHT as usize),
            opacity: 1.0,
        }
    }

    fn load_entry(&mut self, entry: i32) -> Option<Arc<PictLayer>> {
        if entry < 0 {
            return None;
        }

        if let Some(cached) = self.cache.get(&(self.filename.clone()?, entry)) {
            return Some(cached.clone());
        }

        let image = self.s25.as_mut()?.load_image(entry as usize).ok()?;
        let image = Arc::new(PictLayer {
            offset: (image.metadata.offset_x, image.metadata.offset_y),
            image: image.into(),
        });

        self.cache
            .put((self.filename.clone()?, entry), image.clone());

        Some(image)
    }

    fn open_s25<P: AsRef<Path>>(filename: P) -> Option<S25Archive> {
        S25Archive::open(filename).ok()
    }

    fn prefetch_entry(&mut self, filename: &str, entry: i32) -> Option<()> {
        if entry < 0 || self.cache.contains(&(filename.into(), entry)) {
            return None;
        }

        let image = if self
            .filename
            .as_ref()
            .map(|v| v == filename)
            .unwrap_or_default()
        {
            let mut s25 = Self::open_s25(filename)?;
            s25.load_image(entry as usize)
        } else {
            self.s25.as_mut()?.load_image(entry as usize)
        }
        .ok()?;

        let image = Arc::new(PictLayer {
            offset: (image.metadata.offset_x, image.metadata.offset_y),
            image: image.into(),
        });

        self.cache
            .put((self.filename.clone()?, entry), image.clone());

        Some(())
    }

    pub fn load(&mut self, filename: &str, entries: &[i32]) {
        self.filename = Some(filename.into());
        self.s25 = Self::open_s25(filename);

        self.entries = entries
            .iter()
            .copied()
            .filter_map(|e| self.load_entry(e))
            .collect();
    }

    pub fn prefetch(&mut self, filename: &str, entries: &[i32]) {
        for &e in entries {
            self.prefetch_entry(filename, e);
        }
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }
}

impl Renderer<CpuBackend, CpuImageBuffer> for LayerRenderer {
    type Context = ();

    fn render(&mut self, target: &mut CpuImageBuffer, _: &Self::Context) {
        self.framebuffer.clear();

        for e in &self.entries {
            self.framebuffer
                .draw_image(&e.image, (e.offset.0 as isize, e.offset.1 as isize));
        }

        target.draw_image(
            &self.framebuffer.rgba_buffer,
            (0, 0),
            (
                self.framebuffer.width as i32,
                self.framebuffer.height as i32,
            ),
            self.opacity,
        );
    }
}
