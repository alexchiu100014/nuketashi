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
    pub offset: (i32, i32),
    pub opacity: f32,
    pub blur: Option<(usize, usize)>,
    //
    update_flag: bool,
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
            update_flag: false,
            blur: None,
            offset: (0, 0),
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

        self.update_flag = true;
    }

    pub fn prefetch(&mut self, filename: &str, entries: &[i32]) {
        for &e in entries {
            self.prefetch_entry(filename, e);
        }
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
        self.update_flag = true;
    }

    pub fn update(&mut self) {
        if !self.update_flag {
            return;
        }

        self.update_flag = false;
        self.framebuffer.clear();

        for e in &self.entries {
            self.framebuffer.draw_image(
                &e.image,
                (
                    (self.offset.0 + e.offset.0) as isize,
                    (self.offset.1 + e.offset.1) as isize,
                ),
            );
        }

        self.apply_blur();
    }

    fn apply_blur(&mut self) {
        use crate::renderer::cpu::image::{ImageView, ImageViewMut};

        if let Some((rx, ry)) = self.blur {
            let (rx, ry) = (rx as isize, ry as isize);

            // x blur
            for y in 0..self.framebuffer.height {
                for x in 0..self.framebuffer.width {
                    let mut weight = 0;
                    let mut color: [f32; 4] = [0f32; 4];

                    for i in -rx..=rx {
                        weight += 1;

                        if let Some(c) = self.framebuffer.get((x as isize + i) as usize, y) {
                            color[0] += c[0] as f32 / 255.0;
                            color[1] += c[1] as f32 / 255.0;
                            color[2] += c[2] as f32 / 255.0;
                            color[3] += c[3] as f32 / 255.0;
                        }
                    }

                    let weight = 255.0 / weight as f32;

                    if let Some(target) = self.framebuffer.get_mut(x, y) {
                        target[0] = (color[0] * weight) as u8;
                        target[1] = (color[1] * weight) as u8;
                        target[2] = (color[2] * weight) as u8;
                        target[3] = (color[3] * weight) as u8;
                    }
                }
            }
            
            // y blur
            for y in 0..self.framebuffer.height {
                for x in 0..self.framebuffer.width {
                    let mut weight = 0;
                    let mut color: [f32; 4] = [0f32; 4];

                    for i in -ry..=ry {
                        weight += 1;

                        if let Some(c) = self.framebuffer.get(x, (y as isize + i) as usize) {
                            color[0] += c[0] as f32 / 255.0;
                            color[1] += c[1] as f32 / 255.0;
                            color[2] += c[2] as f32 / 255.0;
                            color[3] += c[3] as f32 / 255.0;
                        }
                    }

                    let weight = 255.0 / weight as f32;

                    if let Some(target) = self.framebuffer.get_mut(x, y) {
                        target[0] = (color[0] * weight) as u8;
                        target[1] = (color[1] * weight) as u8;
                        target[2] = (color[2] * weight) as u8;
                        target[3] = (color[3] * weight) as u8;
                    }
                }
            }
        }
    }
}

impl Renderer<CpuBackend, CpuImageBuffer> for LayerRenderer {
    type Context = ();

    fn render(&mut self, target: &mut CpuImageBuffer, _: &Self::Context) {
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
