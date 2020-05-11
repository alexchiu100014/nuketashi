//! Text rendering using RustType.

use crate::constants::FONT_BYTES;
use rusttype::{point, Font, Scale};

use lazy_static::*;

lazy_static! {
    static ref _FONT: Font<'static> = {
        let font = include_bytes!("../../../blob/NUKITASHI_D.WAR/ROUNDED-X-MGENPLUS-1M.TTF");
        Font::try_from_bytes(font).expect("error constructing a Font from bytes")
    };
}

pub fn create_font() -> &'static Font<'static> {
    &*_FONT
}

pub fn write_text_in_box(
    font: &Font,
    font_height: f32,
    text: &str,
    (width, height): (usize, usize),
    mono_buffer: &mut [u8],
) {
    let scale = Scale {
        x: font_height,
        y: font_height,
    };

    let fwidth = width as f32;
    let v_metrics = font.v_metrics(scale);

    // generate a layout
    let layout = font
        .glyphs_for(text.chars())
        .scan((None, 0.0, v_metrics.ascent), |state, g| {
            let last = &mut state.0;
            let x = &mut state.1;
            let y = &mut state.2;

            let g = g.scaled(scale);

            if let Some(last) = last {
                *x += font.pair_kerning(scale, *last, g.id());
            }

            let w = g.h_metrics().advance_width;
            let new_x = *x + w;

            *last = Some(g.id());

            if new_x > fwidth {
                *x = w;
                *y += font_height;
                *last = None;

                Some(g.positioned(point(0.0, *y)))
            } else {
                let old_x = *x;
                *x = new_x;
                Some(g.positioned(point(old_x, *y)))
            }
        });

    for g in layout {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;

                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    mono_buffer[x + y * width] = (v * 255.0).min(255.0) as u8;
                }
            });
        }
    }
}

// from https://gitlab.redox-os.org/redox-os/rusttype/-/blob/master/dev/examples/ascii.rs
#[test]
fn draw_sample_text() {
    use png;
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::Path;

    let font = {
        let font = include_bytes!("../../../blob/NUKITASHI_D.WAR/ROUNDED-X-MGENPLUS-1M.TTF");
        Font::try_from_bytes(font).expect("error constructing a Font from bytes")
    };

    let width = 300;
    let height = 200;

    let path = Path::new(r"./test/harameora.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut buf = vec![0u8; width * height];
    write_text_in_box(
        &font,
        48.0,
        "桐香「ズコバコズコバコ孕めオラ〜♪」Touka \"zuko-bako zuko-bako harame-ora~!\" Touka \"zuko-bako zuko-bako harame-ora~!\"",
        (width, height),
        &mut buf,
    );

    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&buf).unwrap();
}
