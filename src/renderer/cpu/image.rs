#[derive(Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba_buffer: Vec<u8>,
}

use s25::S25Image;

impl From<S25Image> for Image {
    fn from(image: S25Image) -> Self {
        let width = image.metadata.width as usize;
        let height = image.metadata.height as usize;

        let rgba_buffer = image
            .rgba_buffer
            .iter()
            .copied()
            .collect();

        Self {
            width,
            height,
            rgba_buffer,
        }
    }
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            rgba_buffer: vec![0; width * height * 4],
        }
    }

    pub fn clear(&mut self) {
        for i in &mut self.rgba_buffer {
            *i = 0;
        }
    }

    pub fn draw_image(&mut self, image: &Image, (x, y): (isize, isize)) {
        self.draw_image_buffer(&image.rgba_buffer, (x, y), (image.width, image.height));
    }

    pub fn draw_image_buffer(
        &mut self,
        buffer: &[u8],
        (x, y): (isize, isize),
        (width, height): (usize, usize),
    ) {
        assert_eq!(buffer.len(), width * height * 4);
        
        for dx in 0..width {
            for dy in 0..height {
                if let [r, g, b, a] = buffer[(dx as usize + (dy * width) as usize) << 2..][..4] {
                    let px = dx as isize + x;
                    let py = dy as isize + y;

                    if px < 0
                        || self.width <= (px as usize)
                        || py < 0
                        || self.height <= (py as usize)
                        || a == 0
                    {
                        continue;
                    }
                    
                    let rsrc = r as u64;
                    let gsrc = g as u64;
                    let bsrc = b as u64;
                    let asrc = a as u64;

                    if let [tr, tg, tb, ta] =
                        &mut self.rgba_buffer[(px as usize + py as usize * self.width) << 2..][..4]
                    {
                        let rdst = *tr as u64;
                        let gdst = *tg as u64;
                        let bdst = *tb as u64;
                        let adst = *ta as u64;

                        let oa = (255 * asrc + adst * (255 - asrc)) >> 8;
                        if oa == 0 {
                            *ta = 0;
                            continue;
                        }

                        let or = (255 * rsrc * asrc + rdst * adst * (255 - asrc)) / oa;
                        let og = (255 * gsrc * asrc + gdst * adst * (255 - asrc)) / oa;
                        let ob = (255 * bsrc * asrc + bdst * adst * (255 - asrc)) / oa;

                        *tr = (or >> 8) as u8;
                        *tg = (og >> 8) as u8;
                        *tb = (ob >> 8) as u8;
                        *ta = oa as u8;
                    }
                }
            }
        }
    }
}
