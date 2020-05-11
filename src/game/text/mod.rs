//! Text rendering using RustType.

use crate::constants::FONT_BYTES;
use rusttype::{point, Font, Scale};

pub fn write_text_in_box(
    font: &Font,
    text: &str,
    (width, height): (usize, usize),
    mono_buffer: &mut [u8],
) {
    let font_height = 24.0;
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
            let next = g.positioned(point(*x, *y));

            let new_x = *x + w;
            let gw = next
                .pixel_bounding_box()
                .map(|r| (r.max - r.min).x)
                .unwrap_or(0) as f32;

            if (new_x + gw) >= fwidth {
                *last = None;
                *x = 0.0;
                *y += font_height;
            } else {
                *x = new_x;
                *last = Some(next.id());
            }

            Some(next)
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

    let width = 200;
    let height = 100;

    let path = Path::new(r"./test/harameora.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut buf = vec![0u8; width * height];
    write_text_in_box(
        &font,
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
