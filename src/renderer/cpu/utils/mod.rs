use crate::renderer::cpu::image::{ImageView, ImageViewMut};

pub fn alpha_blend<I1, I2>(src: &I1, dest: &mut I2, (x, y): (isize, isize), opacity: f32)
where
    I1: ImageView,
    I2: ImageViewMut,
{
    let opacity = (opacity.min(1.0).max(0.0) * 256.0) as u32;

    for dy in ((-y).max(0) as usize)
        ..src
            .get_height()
            .min((-y + dest.get_height() as isize) as usize)
    {
        let py = (dy as isize + y) as usize;

        for dx in ((-x).max(0) as usize)
            ..src
                .get_width()
                .min((-x + dest.get_width() as isize) as usize)
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
                } else if *da == 0 {
                    *dr = *sr;
                    *dg = *sg;
                    *db = *sb;
                    *da = *sa;
                    continue;
                }

                let rsrc = *sr as u32;
                let gsrc = *sg as u32;
                let bsrc = *sb as u32;
                let asrc = (*sa as u32 * opacity) >> 8;

                if asrc == 255 {
                    *dr = rsrc as u8;
                    *dg = gsrc as u8;
                    *db = bsrc as u8;
                    *da = 255;
                    continue;
                }

                let rdst = *dr as u32;
                let gdst = *dg as u32;
                let bdst = *db as u32;
                let adst = *da as u32;

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


pub fn alpha_blend_colored<I1, I2>(src: &I1, dest: &mut I2, (x, y): (isize, isize), opacity: f32, tint: [u8; 3])
where
    I1: ImageView,
    I2: ImageViewMut,
{
    let opacity = (opacity.min(1.0).max(0.0) * 256.0) as u32;

    for dy in ((-y).max(0) as usize)
        ..src
            .get_height()
            .min((-y + dest.get_height() as isize) as usize)
    {
        let py = (dy as isize + y) as usize;

        for dx in ((-x).max(0) as usize)
            ..src
                .get_width()
                .min((-x + dest.get_width() as isize) as usize)
        {
            let px = (dx as isize + x) as usize;

            if let (Some([dr, dg, db, da]), Some([_, _, _, sa])) =
                (dest.get_mut(px, py), src.get(dx, dy))
            {
                if *sa == 0 {
                    continue;
                } else if *sa == 255 && opacity == 255 {
                    *dr = tint[0];
                    *dg = tint[1];
                    *db = tint[2];
                    *da = 255;
                    continue;
                } else if *da == 0 {
                    *dr = tint[0];
                    *dg = tint[1];
                    *db = tint[2];
                    *da = *sa;
                    continue;
                }

                let rsrc = tint[0] as u32;
                let gsrc = tint[1] as u32;
                let bsrc = tint[2] as u32;
                let asrc = (*sa as u32 * opacity) >> 8;

                if asrc == 255 {
                    *dr = rsrc as u8;
                    *dg = gsrc as u8;
                    *db = bsrc as u8;
                    *da = 255;
                    continue;
                }

                let rdst = *dr as u32;
                let gdst = *dg as u32;
                let bdst = *db as u32;
                let adst = *da as u32;

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

use crate::renderer::cpu::image::{ImageSlice, ImageSliceMut};

pub fn alpha_blend_simd(
    src: &ImageSlice,
    dest: &mut ImageSliceMut,
    (x, y): (isize, isize),
    opacity: f32,
) {
    for dy in ((-y).max(0) as usize)
        ..src
            .get_height()
            .min((-y + dest.get_height() as isize) as usize)
    {
        let py = (dy as isize + y) as usize;

        let origin = (-x).max(0) as usize;

        let width = src
            .get_width()
            .min((-x + dest.get_width() as isize) as usize);

        let offset_src = dy * src.get_width() << 2;
        let offset_dst = py * dest.get_width() << 2;

        unsafe {
            alpha_blend_simd_scanline(
                &src.rgba_buffer[offset_src..][(origin << 2)..(width << 2)],
                &mut dest.rgba_buffer[offset_dst..][(((origin as isize + x) as usize) << 2)
                    ..(((width as isize + x) as usize) << 2)],
                opacity,
            );
        }
    }
}

unsafe fn alpha_blend_simd_scanline(src: &[u8], dst: &mut [u8], opacity: f32) {
    use std::arch::x86_64::*;

    assert_eq!(src.len(), dst.len());
    assert_eq!(src.len() & 0x03, 0);
    assert_eq!(dst.len() & 0x03, 0);

    let opacity: __m256 = std::mem::transmute([opacity; 8]);
    let saturated: __m256 = std::mem::transmute([1.0f32; 8]);
    let u8max: __m256 = std::mem::transmute([255.0f32; 8]);

    for i in 0..((src.len() >> 4) + 1) {
        let offset = i << 4;

        if src.len() <= offset {
            break;
        }

        let mut rsrc = [0f32; 8];
        let mut gsrc = [0f32; 8];
        let mut bsrc = [0f32; 8];
        let mut asrc = [0f32; 8];

        let mut rdst = [0f32; 8];
        let mut gdst = [0f32; 8];
        let mut bdst = [0f32; 8];
        let mut adst = [0f32; 8];

        for j in 0..8 {
            let j4 = j << 2;

            if src.len() <= (offset + j4) {
                break;
            }

            rsrc[j] = src[offset + j4] as f32;
            gsrc[j] = src[offset + j4 + 1] as f32;
            bsrc[j] = src[offset + j4 + 2] as f32;
            asrc[j] = src[offset + j4 + 3] as f32;

            rdst[j] = dst[offset + j4] as f32;
            gdst[j] = dst[offset + j4 + 1] as f32;
            bdst[j] = dst[offset + j4 + 2] as f32;
            adst[j] = dst[offset + j4 + 3] as f32;
        }

        let rsrc: __m256 = std::mem::transmute(rsrc);
        let gsrc: __m256 = std::mem::transmute(gsrc);
        let bsrc: __m256 = std::mem::transmute(bsrc);
        let asrc: __m256 = std::mem::transmute(asrc);
        let rdst: __m256 = std::mem::transmute(rdst);
        let gdst: __m256 = std::mem::transmute(gdst);
        let bdst: __m256 = std::mem::transmute(bdst);
        let adst: __m256 = std::mem::transmute(adst);

        let rsrc = _mm256_div_ps(rsrc, u8max);
        let gsrc = _mm256_div_ps(gsrc, u8max);
        let bsrc = _mm256_div_ps(bsrc, u8max);
        let asrc = _mm256_div_ps(asrc, u8max);
        let rdst = _mm256_div_ps(rdst, u8max);
        let gdst = _mm256_div_ps(gdst, u8max);
        let bdst = _mm256_div_ps(bdst, u8max);
        let adst = _mm256_div_ps(adst, u8max);

        let asrc = _mm256_mul_ps(asrc, opacity);
        let asrc_inv = _mm256_sub_ps(saturated, asrc);

        let oa = _mm256_add_ps(asrc, _mm256_mul_ps(adst, asrc_inv));

        let or = _mm256_div_ps(
            _mm256_add_ps(
                _mm256_mul_ps(rsrc, asrc),
                _mm256_mul_ps(rdst, _mm256_mul_ps(adst, asrc_inv)),
            ),
            oa,
        );
        let og = _mm256_div_ps(
            _mm256_add_ps(
                _mm256_mul_ps(gsrc, asrc),
                _mm256_mul_ps(gdst, _mm256_mul_ps(adst, asrc_inv)),
            ),
            oa,
        );
        let ob = _mm256_div_ps(
            _mm256_add_ps(
                _mm256_mul_ps(bsrc, asrc),
                _mm256_mul_ps(bdst, _mm256_mul_ps(adst, asrc_inv)),
            ),
            oa,
        );

        let or = _mm256_mul_ps(or, u8max);
        let og = _mm256_mul_ps(og, u8max);
        let ob = _mm256_mul_ps(ob, u8max);
        let oa = _mm256_mul_ps(oa, u8max);

        let or: [f32; 8] = std::mem::transmute(or);
        let og: [f32; 8] = std::mem::transmute(og);
        let ob: [f32; 8] = std::mem::transmute(ob);
        let oa: [f32; 8] = std::mem::transmute(oa);

        for j in 0..8 {
            let j4 = j << 2;

            if src.len() <= (offset + j4) {
                break;
            }

            dst[offset + j4] = or[j] as u8;
            dst[offset + j4 + 1] = og[j] as u8;
            dst[offset + j4 + 2] = ob[j] as u8;
            dst[offset + j4 + 3] = oa[j] as u8;
        }
    }
}
