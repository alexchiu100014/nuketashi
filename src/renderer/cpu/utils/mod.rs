use crate::renderer::cpu::image::{ImageView, ImageViewMut};

pub fn alpha_blend<I1, I2>(src: &I1, dest: &mut I2, (x, y): (isize, isize), opacity: f32)
where
    I1: ImageView,
    I2: ImageViewMut,
{
    let opacity = (opacity.min(1.0).max(0.0) * 256.0) as u64;

    for dy in 0..src.get_height() {
        let py = dy as isize + y;

        if py < 0 || dest.get_height() <= py as usize  {
            continue;
        }

        let py = py as usize;

        for dx in 0..src.get_width() {
            let px = dx as isize + x;

            if px < 0 || dest.get_width() <= px as usize {
                continue;
            }

            let px = px as usize;

            if let (Some([dr, dg, db, da]), Some([sr, sg, sb, sa])) =
                (dest.get_mut(px, py), src.get(dx, dy))
            {
                let rsrc = *sr as u64;
                let gsrc = *sg as u64;
                let bsrc = *sb as u64;
                let asrc = (*sa as u64 * opacity) >> 8;

                let rdst = *dr as u64;
                let gdst = *dg as u64;
                let bdst = *db as u64;
                let adst = *da as u64;

                let oa = (255 * asrc + adst * (255 - asrc)) >> 8;
                if oa == 0 {
                    *da = 0;
                    continue;
                }

                let or = (255 * rsrc * asrc + rdst * adst * (255 - asrc)) / oa;
                let og = (255 * gsrc * asrc + gdst * adst * (255 - asrc)) / oa;
                let ob = (255 * bsrc * asrc + bdst * adst * (255 - asrc)) / oa;

                *dr = (or >> 8) as u8;
                *dg = (og >> 8) as u8;
                *db = (ob >> 8) as u8;
                *da = oa as u8;
            }
        }
    }
}
