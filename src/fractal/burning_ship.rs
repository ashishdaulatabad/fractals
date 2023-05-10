use crate::fractal::{Block, IterationType};
use crate::utils::complex::Complex64;
use core::arch::x86_64;

use super::{FSignature, InstructionSet, Precision};

pub fn fn_(prec: Precision, ins: InstructionSet) -> FSignature {
    match prec {
        Precision::F64 => match ins {
            InstructionSet::AVX => burning_ship_simd256,
            InstructionSet::SSE => burning_ship_simd,
            InstructionSet::None => burning_ship,
        },
        Precision::F32 => match ins {
            InstructionSet::AVX => burning_shipf32_simd256,
            InstructionSet::SSE => burning_shipf32_simd,
            InstructionSet::None => burning_shipf32,
        },
    }
}

pub fn burning_ship(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    _: i32,
    _: i32,
) -> Block {
    let h = height as f64;

    // Perform operation on section of image
    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in 0..width {
            let x = xcoord as f64;
            let mut iterations = 1;
            let a = Complex64::new(x / h * 2.0 - 2.0, y / h * 2.0 - 0.5);
            let mut z = a;

            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z.real = if z.real > 0.0 { -z.real } else { z.real };
                z.img = if z.img > 0.0 { z.img } else { -z.img };

                z = z.ipow(pow);
                z += a;
                iterations += 1;
            }

            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse")]
#[target_feature(enable = "sse2")]
pub unsafe fn burning_ship_simd(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    _: i32,
    _: i32,
) -> Block {
    let h = height as f64;
    let mut iterations;
    let rem = width & 1;

    // Perform operation on section of image
    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in (0..width).step_by(2) {
            let x = xcoord as f64;
            let ax = x86_64::_mm_set_pd(x / h * 2.0 - 2.0, (x + 1.0) / h * 2.0 - 2.0);
            let ay = x86_64::_mm_set1_pd(y / h * 2.0 - 0.5);

            let mut zx = x86_64::_mm_set1_pd(0.0);
            let mut zy = x86_64::_mm_set1_pd(0.0);

            let iter_cmp = x86_64::_mm_set1_epi64x(max_iterations as i64);
            let compare = x86_64::_mm_set1_pd(4.0);
            iterations = x86_64::_mm_set1_epi64x(0);

            loop {
                let tmpzx = x86_64::_mm_mul_pd(zx, x86_64::_mm_set1_pd(-1.0));
                zx = x86_64::_mm_blendv_pd(tmpzx, zx, x86_64::_mm_cmplt_pd(tmpzx, zx));
                let tmpzy = x86_64::_mm_mul_pd(zy, x86_64::_mm_set1_pd(-1.0));
                zy = x86_64::_mm_blendv_pd(tmpzy, zy, x86_64::_mm_cmplt_pd(zy, tmpzy));

                for _ in 1..pow {
                    let tmp =
                        x86_64::_mm_sub_pd(x86_64::_mm_mul_pd(zx, zx), x86_64::_mm_mul_pd(zy, zy));
                    let zxy = x86_64::_mm_mul_pd(zx, zy);
                    zy = x86_64::_mm_add_pd(zxy, zxy);
                    zx = tmp;
                }

                zx = x86_64::_mm_add_pd(zx, ax);
                zy = x86_64::_mm_add_pd(zy, ay);
                let mg = x86_64::_mm_add_pd(x86_64::_mm_mul_pd(zx, zx), x86_64::_mm_mul_pd(zy, zy));
                let cmp_mg = x86_64::_mm_cmplt_pd(mg, compare);
                let value = x86_64::_mm_movemask_pd(cmp_mg) as i64;
                if value == 0 {
                    break;
                }
                iterations = x86_64::_mm_add_epi64(
                    iterations,
                    x86_64::_mm_set_epi64x((value & 2) >> 1, value & 1),
                );
                let cmp_iter = x86_64::_mm_cmpeq_epi64(iterations, iter_cmp);
                let msk = x86_64::_mm_movemask_epi8(cmp_iter);
                if msk > 0 {
                    break;
                }
            }

            let mut iter: [i64; 2] = [0; 2];
            x86_64::_mm_storeu_epi64(iter.as_mut_ptr(), iterations);
            block[ycoord - hstart][xcoord] = iter[1] as IterationType;
            block[ycoord - hstart][xcoord + 1] = iter[0] as IterationType;
        }
        if rem & 1 > 0 {
            let x = (width - 1) as f64;
            let mut iterations = 1;
            let a = Complex64::new(x / h * 2.0 - 2.0, y / h * 2.0 - 0.5);
            let mut z = a;

            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z.real = if z.real > 0.0 { -z.real } else { z.real };
                z.img = if z.img > 0.0 { z.img } else { -z.img };

                z = z.ipow(pow);
                z += a;
                iterations += 1;
            }

            block[ycoord - hstart][width - 1] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
#[target_feature(enable = "avx2")]
pub unsafe fn burning_ship_simd256(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    _: i32,
    _: i32,
) -> Block {
    let h = height as f64;
    let mut iter: [i64; 4] = [0; 4];
    let size = width;
    let rem = size & 3;

    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in (0..(width - rem)).step_by(4) {
            let x = xcoord as f64;
            // print!("start {} ", ycoord);
            let ax = x86_64::_mm256_set_pd(
                x / h * 2.0 - 2.0,
                (x + 1.0) / h * 2.0 - 2.0,
                (x + 2.0) / h * 2.0 - 2.0,
                (x + 3.0) / h * 2.0 - 2.0,
            );
            let ay = x86_64::_mm256_set1_pd(y / h * 2.0 - 0.5);
            let mut iterations = x86_64::_mm256_set1_epi64x(0);
            let mut zx = x86_64::_mm256_set1_pd(0.0);
            let mut zy = x86_64::_mm256_set1_pd(0.0);

            let iter_cmp = x86_64::_mm256_set1_epi64x(max_iterations as i64);
            let compare = x86_64::_mm256_set1_pd(4.0);

            loop {
                // Set all real values to negative
                let tmpzx = x86_64::_mm256_mul_pd(zx, x86_64::_mm256_set1_pd(-1.0));
                zx = x86_64::_mm256_blendv_pd(tmpzx, zx, x86_64::_mm256_cmp_pd::<17>(tmpzx, zx));
                // Set all img values to positive
                let tmpzy = x86_64::_mm256_mul_pd(zy, x86_64::_mm256_set1_pd(-1.0));
                zy = x86_64::_mm256_blendv_pd(tmpzy, zy, x86_64::_mm256_cmp_pd::<17>(zy, tmpzy));

                for _ in 1..pow {
                    let tmp = x86_64::_mm256_sub_pd(
                        x86_64::_mm256_mul_pd(zx, zx),
                        x86_64::_mm256_mul_pd(zy, zy),
                    );
                    let zxy = x86_64::_mm256_mul_pd(zx, zy);
                    zy = x86_64::_mm256_add_pd(zxy, zxy);
                    zx = tmp;
                }

                zx = x86_64::_mm256_add_pd(zx, ax);
                zy = x86_64::_mm256_add_pd(zy, ay);
                let mg = x86_64::_mm256_add_pd(
                    x86_64::_mm256_mul_pd(zx, zx),
                    x86_64::_mm256_mul_pd(zy, zy),
                );
                // x86_64::_CMP_LT_OQ = 0x11 = 17
                // x86_64::_CMP_LT_OS = 0x1 = 1
                let cmp_mg = x86_64::_mm256_cmp_pd::<17>(mg, compare);
                let value = x86_64::_mm256_movemask_pd(cmp_mg) as i64;
                if value == 0 {
                    break;
                }
                iterations = x86_64::_mm256_add_epi64(
                    iterations,
                    x86_64::_mm256_set_epi64x(
                        (value & 8) >> 3,
                        (value & 4) >> 2,
                        (value & 2) >> 1,
                        value & 1,
                    ),
                );

                let cmp_iter = x86_64::_mm256_cmpeq_epi64(iter_cmp, iterations);
                let msk = x86_64::_mm256_movemask_epi8(cmp_iter);
                if msk != 0 {
                    break;
                }
            }
            x86_64::_mm256_storeu_epi64(iter.as_mut_ptr(), iterations);
            block[ycoord - hstart][xcoord] = iter[3] as IterationType;
            block[ycoord - hstart][xcoord + 1] = iter[2] as IterationType;
            block[ycoord - hstart][xcoord + 2] = iter[1] as IterationType;
            block[ycoord - hstart][xcoord + 3] = iter[0] as IterationType;
        }

        for xcoord in (width - rem)..width {
            let x = xcoord as f64;
            let mut iterations = 1;
            let a = Complex64::new(x / h * 2.0 - 2.0, y / h * 2.0 - 0.5);
            let mut z = a;

            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z.real = if z.real > 0.0 { -z.real } else { z.real };
                z.img = if z.img > 0.0 { z.img } else { -z.img };

                z = z.ipow(pow);
                z += a;
                iterations += 1;
            }

            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

pub fn burning_shipf32(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    _: i32,
    _: i32,
) -> Block {
    let h = height as f64;

    // Perform operation on section of image
    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in 0..width {
            let x = xcoord as f64;
            let mut iterations = 1;
            let a = Complex64::new(x / h * 2.0 - 2.0, y / h * 2.0 - 0.5);
            let mut z = a;

            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z.real = if z.real > 0.0 { -z.real } else { z.real };
                z.img = if z.img > 0.0 { z.img } else { -z.img };

                z = z.ipow(pow);
                z += a;
                iterations += 1;
            }

            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse")]
#[target_feature(enable = "sse2")]
pub unsafe fn burning_shipf32_simd(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    _: i32,
    _: i32,
) -> Block {
    let h = height as f32;

    let mut iterations;

    // Perform operation on section of image
    for ycoord in hstart..hend {
        let y = ycoord as f32;
        for xcoord in (0..width).step_by(4) {
            let x = xcoord as f32;
            let ax = x86_64::_mm_set_ps(
                x / h * 2.0 - 2.0,
                (x + 1.0) / h * 2.0 - 2.0,
                (x + 2.0) / h * 2.0 - 2.0,
                (x + 3.0) / h * 2.0 - 2.0,
            );
            let ay = x86_64::_mm_set1_ps(y / h * 2.0 - 0.5);

            let mut zx = x86_64::_mm_set1_ps(0.0);
            let mut zy = x86_64::_mm_set1_ps(0.0);

            let iter_cmp = x86_64::_mm_set1_epi32(max_iterations as i32);
            let compare = x86_64::_mm_set1_ps(4.0);
            iterations = x86_64::_mm_set1_epi32(0);

            loop {
                let tmpzx = x86_64::_mm_mul_ps(zx, x86_64::_mm_set1_ps(-1.0));
                zx = x86_64::_mm_blendv_ps(tmpzx, zx, x86_64::_mm_cmplt_ps(tmpzx, zx));
                let tmpzy = x86_64::_mm_mul_ps(zy, x86_64::_mm_set1_ps(-1.0));
                zy = x86_64::_mm_blendv_ps(tmpzy, zy, x86_64::_mm_cmplt_ps(zy, tmpzy));

                for _ in 1..pow {
                    let tmp =
                        x86_64::_mm_sub_ps(x86_64::_mm_mul_ps(zx, zx), x86_64::_mm_mul_ps(zy, zy));
                    let zxy = x86_64::_mm_mul_ps(zx, zy);
                    zy = x86_64::_mm_add_ps(zxy, zxy);
                    zx = tmp;
                }

                zx = x86_64::_mm_add_ps(zx, ax);
                zy = x86_64::_mm_add_ps(zy, ay);
                let mg = x86_64::_mm_add_ps(x86_64::_mm_mul_ps(zx, zx), x86_64::_mm_mul_ps(zy, zy));
                let cmp_mg = x86_64::_mm_cmplt_ps(mg, compare);
                let value = x86_64::_mm_movemask_ps(cmp_mg) as i32;
                if value == 0 {
                    break;
                }
                iterations = x86_64::_mm_add_epi32(
                    iterations,
                    x86_64::_mm_set_epi32(
                        value & 1,
                        (value & 2) >> 1,
                        (value & 4) >> 2,
                        (value & 8) >> 3,
                    ),
                );
                let cmp_iter = x86_64::_mm_cmpeq_epi32(iterations, iter_cmp);
                let msk = x86_64::_mm_movemask_epi8(cmp_iter);
                if msk > 0 {
                    break;
                }
            }

            x86_64::_mm_storeu_epi32(
                block[ycoord - hstart][xcoord..].as_mut_ptr() as *mut i32,
                iterations,
            );
        }
    }
    (hstart, hend)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx,avx2")]
pub unsafe fn burning_shipf32_simd256(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    _: i32,
    _: i32,
) -> Block {
    let h = height as f32;

    let mut iterations;
    let size = width;
    let rem = size & 7;

    for ycoord in hstart..hend {
        let y = ycoord as f32;
        for xcoord in (0..(width - rem)).step_by(8) {
            let x = xcoord as f32;
            // print!("start {} ", ycoord);
            let ax = x86_64::_mm256_set_ps(
                x / h * 2.0 - 2.0,
                (x + 1.0) / h * 2.0 - 2.0,
                (x + 2.0) / h * 2.0 - 2.0,
                (x + 3.0) / h * 2.0 - 2.0,
                (x + 4.0) / h * 2.0 - 2.0,
                (x + 5.0) / h * 2.0 - 2.0,
                (x + 6.0) / h * 2.0 - 2.0,
                (x + 7.0) / h * 2.0 - 2.0,
            );
            let ay = x86_64::_mm256_set1_ps(y / h * 2.0 - 0.5);
            iterations = x86_64::_mm256_set1_epi32(0);
            let mut zx = x86_64::_mm256_set1_ps(0.0);
            let mut zy = x86_64::_mm256_set1_ps(0.0);

            let iter_cmp = x86_64::_mm256_set1_epi32(max_iterations as i32);
            let compare = x86_64::_mm256_set1_ps(4.0);

            loop {
                // Set all real values to negative
                let tmpzx = x86_64::_mm256_mul_ps(zx, x86_64::_mm256_set1_ps(-1.0));
                zx = x86_64::_mm256_blendv_ps(tmpzx, zx, x86_64::_mm256_cmp_ps::<17>(tmpzx, zx));
                // Set all img values to positive
                let tmpzy = x86_64::_mm256_mul_ps(zy, x86_64::_mm256_set1_ps(-1.0));
                zy = x86_64::_mm256_blendv_ps(tmpzy, zy, x86_64::_mm256_cmp_ps::<17>(zy, tmpzy));

                for _ in 1..pow {
                    let tmp = x86_64::_mm256_sub_ps(
                        x86_64::_mm256_mul_ps(zx, zx),
                        x86_64::_mm256_mul_ps(zy, zy),
                    );
                    let zxy = x86_64::_mm256_mul_ps(zx, zy);
                    zy = x86_64::_mm256_add_ps(zxy, zxy);
                    zx = tmp;
                }

                zx = x86_64::_mm256_add_ps(zx, ax);
                zy = x86_64::_mm256_add_ps(zy, ay);
                let mg = x86_64::_mm256_add_ps(
                    x86_64::_mm256_mul_ps(zx, zx),
                    x86_64::_mm256_mul_ps(zy, zy),
                );
                // x86_64::_CMP_LT_OQ = 0x11 = 17
                // x86_64::_CMP_LT_OS = 0x1 = 1
                let cmp_mg = x86_64::_mm256_cmp_ps::<17>(mg, compare);
                let value = x86_64::_mm256_movemask_ps(cmp_mg) as i32;
                if value == 0 {
                    break;
                }
                iterations = x86_64::_mm256_add_epi32(
                    iterations,
                    x86_64::_mm256_set_epi32(
                        value & 1,
                        (value & 2) >> 1,
                        (value & 4) >> 2,
                        (value & 8) >> 3,
                        (value & 16) >> 4,
                        (value & 32) >> 5,
                        (value & 64) >> 6,
                        (value & 128) >> 7,
                    ),
                );

                let cmp_iter = x86_64::_mm256_cmpeq_epi32(iter_cmp, iterations);
                let msk = x86_64::_mm256_movemask_epi8(cmp_iter);
                if msk != 0 {
                    break;
                }
            }
            x86_64::_mm256_storeu_epi32(
                block[ycoord - hstart][xcoord..].as_mut_ptr() as *mut i32,
                iterations,
            );
        }

        for xcoord in (width - rem)..width {
            let x = xcoord as f64;
            let hh = h as f64;
            let yy = y as f64;
            let mut iterations = 1;
            let a = Complex64::new(x / hh * 2.0 - 2.0, yy / hh * 2.0 - 0.5);
            let mut z = a;

            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z.real = if z.real > 0.0 { -z.real } else { z.real };
                z.img = if z.img > 0.0 { z.img } else { -z.img };

                z = z.ipow(pow);
                z += a;
                iterations += 1;
            }

            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}
