#[derive(Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba_buffer: Vec<f32>,
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
            .map(|v| v as f32 / 255.0)
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
            rgba_buffer: vec![0.0; width * height * 4],
        }
    }

    pub fn clear(&mut self) {
        for i in &mut self.rgba_buffer {
            *i = 0.0;
        }
    }

    pub fn draw_image(&mut self, image: &Image, (x, y): (isize, isize)) {
        self.draw_image_buffer(&image.rgba_buffer, (x, y), (image.width, image.height));
    }

    pub fn draw_image_buffer(
        &mut self,
        buffer: &[f32],
        (x, y): (isize, isize),
        (width, height): (usize, usize),
    ) {
        for dx in 0..width {
            for dy in 0..height {
                if let [r, g, b, a] = buffer[dx as usize + (dy * height) as usize..][0..4] {
                    let px = dx as isize + x;
                    let py = dy as isize + y;

                    let r = r.min(1.0).max(0.0);
                    let g = g.min(1.0).max(0.0);
                    let b = b.min(1.0).max(0.0);
                    let a = a.min(1.0).max(0.0);

                    if px < 0
                        || self.width <= (px as usize)
                        || py < 0
                        || self.height <= (py as usize)
                    {
                        continue;
                    }

                    if let [tr, tg, tb, ta] =
                        &mut self.rgba_buffer[px as usize + py as usize * self.height..][0..4]
                    {
                        let rsrc = *tr;
                        let gsrc = *tg;
                        let bsrc = *tb;
                        let asrc = *ta;

                        let oa = asrc + a * (1.0 - asrc);
                        if oa.abs() <= f32::EPSILON {
                            *tr = 0.0;
                            *tg = 0.0;
                            *tb = 0.0;
                            *ta = 0.0;
                            continue;
                        }

                        let or = (rsrc * asrc + r * a * (1.0 - asrc)) / oa;
                        let og = (gsrc * asrc + g * a * (1.0 - asrc)) / oa;
                        let ob = (bsrc * asrc + b * a * (1.0 - asrc)) / oa;

                        *tr = or;
                        *tg = og;
                        *tb = ob;
                        *ta = oa;
                    }
                }
            }
        }
    }
}
