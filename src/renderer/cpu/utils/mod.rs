use crate::renderer::cpu::image::{ImageView, ImageViewMut};

pub fn alpha_blend<I1, I2>(src: &I1, dest: &mut I2, (x, y): (isize, isize), opacity: f32)
where
    I1: ImageView,
    I2: ImageViewMut,
{
    let opacity = (opacity.min(1.0).max(0.0) * 256.0) as u64;

    for dy in ((-y).max(0) as usize)
        ..src
            .get_height()
            .min((-y + dest.get_height() as isize - 1) as usize)
    {
        let py = (dy as isize + y) as usize;

        for dx in ((-x).max(0) as usize)
            ..src
                .get_width()
                .min((-x + dest.get_width() as isize - 1) as usize)
        {
            let px = (dx as isize + x) as usize;

            if let (Some([dr, dg, db, da]), Some([sr, sg, sb, sa])) =
                (dest.get_mut(px, py), src.get(dx, dy))
            {
                if *sa == 0 {
                    continue;
                } else if *sa == 255 && opacity == 255 {
                    *dr = *sr;
                    *dg = *sg;
                    *db = *sb;
                    *da = 255;
                    continue;
                }

                let rsrc = *sr as u64;
                let gsrc = *sg as u64;
                let bsrc = *sb as u64;
                let asrc = (*sa as u64 * opacity) >> 8;

                if asrc == 255 {
                    *dr = *sr;
                    *dg = *sg;
                    *db = *sb;
                    *da = 255;
                    continue;
                }

                let rdst = *dr as u64;
                let gdst = *dg as u64;
                let bdst = *db as u64;
                let adst = *da as u64;

                let oa = (255 * asrc + adst * (255 - asrc)) >> 8;
                if oa == 0 {
                    *da = 0;
                    continue;
                }

                let or = ((rsrc * asrc) * 255 + rdst * adst * (255 - asrc)) / oa;
                let og = ((gsrc * asrc) * 255 + gdst * adst * (255 - asrc)) / oa;
                let ob = ((bsrc * asrc) * 255 + bdst * adst * (255 - asrc)) / oa;

                *dr = (or >> 8) as u8;
                *dg = (og >> 8) as u8;
                *db = (ob >> 8) as u8;

                *da = oa as u8;
            }
        }
    }
}
