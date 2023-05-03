use crate::complex;
use core::arch::x86_64;

use crate::fractal::{Block, IterationType};

use super::{FSignature, InstructionSet, Precision};

// const xfpos: f64 = -0.7777;
// const yfpos: f64 = 0.2;

pub fn fn_(prec: Precision, ins: InstructionSet) -> FSignature {
    match prec {
        Precision::F64 => match ins {
            InstructionSet::AVX => julia_simd256,
            InstructionSet::SSE => julia_simd,
            InstructionSet::None => julia,
        },
        Precision::F32 => match ins {
            InstructionSet::AVX => juliaf32_simd256,
            InstructionSet::SSE => juliaf32_simd,
            InstructionSet::None => juliaf32,
        },
    }
}

pub fn julia(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    xpos: i32,
    ypos: i32,
) -> Block {
    let h = height as f64;
    let xfpos = xpos as f64 / h * 2.0 - 1.8;
    let yfpos = ypos as f64 / h * 2.0 - 1.0;
    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in 0..width {
            let x = xcoord as f64;
            let mut z = complex::Complex::new(x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let a = complex::Complex::new(xfpos, yfpos);
            let mut iterations = 0;
            while iterations < max_iterations && z.abs_sq() < 4.0 {
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
#[target_feature(enable = "avx2")]
pub unsafe fn julia_simd(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    xpos: i32,
    ypos: i32,
) -> Block {
    let h = height as f64;
    let mut iterations;
    let rem = width & 1;

    let xfpos = xpos as f64 / h * 2.0 - 1.8;
    let yfpos = ypos as f64 / h * 2.0 - 1.0;
    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in (0..width).step_by(2) {
            let x = xcoord as f64;

            let ax = x86_64::_mm_set1_pd(xfpos);
            let ay = x86_64::_mm_set1_pd(yfpos);
            iterations = x86_64::_mm_set1_epi64x(1);
            let mut zx = x86_64::_mm_set_pd(x / h * 2.0 - 1.8, (x + 1.0) / h * 2.0 - 1.8);
            let mut zy = x86_64::_mm_set1_pd(y / h * 2.0 - 1.0);

            let iter_cmp = x86_64::_mm_set1_epi64x(max_iterations as i64);
            let compare = x86_64::_mm_set1_pd(4.0);

            loop {
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
        if rem & 1 == 1 {
            let x = (hend - 1) as f64;
            let mut z = complex::Complex::new(x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let a = complex::Complex::new(xfpos, yfpos);
            let mut iterations = 0;
            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z = z.ipow(pow);
                z += a;
                iterations += 1;
            }
            block[ycoord][hend - 1 - hstart] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn julia_simd256(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    xpos: i32,
    ypos: i32,
) -> Block {
    let h = height as f64;

    let xfpos = xpos as f64 / h * 2.0 - 1.8;
    let yfpos = ypos as f64 / h * 2.0 - 1.0;
    let mut iterations;
    let mut iter: [i64; 4] = [0; 4];
    let size = width;
    let rem = size & 3;

    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in (0..(width - rem)).step_by(4) {
            let x = xcoord as f64;
            // print!("start {} ", ycoord);
            let ax = x86_64::_mm256_set1_pd(xfpos);
            let ay = x86_64::_mm256_set1_pd(yfpos);
            iterations = x86_64::_mm256_set1_epi64x(1);
            let mut zx = x86_64::_mm256_set_pd(
                x / h * 2.0 - 1.8,
                (x + 1.0) / h * 2.0 - 1.8,
                (x + 2.0) / h * 2.0 - 1.8,
                (x + 3.0) / h * 2.0 - 1.8,
            );
            let mut zy = x86_64::_mm256_set1_pd(y / h * 2.0 - 1.0);

            let iter_cmp = x86_64::_mm256_set1_epi64x(max_iterations as i64);
            let compare = x86_64::_mm256_set1_pd(4.0);

            loop {
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
            let a = complex::Complex::new(x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let mut z = a;
            let mut iterations = 1;
            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z = z.ipow(pow);
                z += a;
                iterations += 1;
            }
            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

pub fn juliaf32(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    xpos: i32,
    ypos: i32,
) -> Block {
    let h = height as f32;
    println!("{} {}", width, height);
    let xfpos = xpos as f32 / h * 2.0 - 1.8;
    let yfpos = ypos as f32 / h * 2.0 - 1.0;
    for ycoord in hstart..hend {
        let y = ycoord as f32;
        for xcoord in 0..width {
            let x = xcoord as f32;
            let mut z = (x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let a = (xfpos, yfpos);
            let mut iterations = 0;
            while iterations < max_iterations && z.0 * z.0 + z.1 * z.1 < 4.0 {
                z = (z.0 * z.0 - z.1 * z.1, 2.0 * z.0 * z.1);
                z = (z.0 + a.0, z.1 + a.1);
                iterations += 1;
            }
            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn juliaf32_simd(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    xpos: i32,
    ypos: i32,
) -> Block {
    let h = height as f32;
    let mut iterations;
    let rem = width & 3;

    let xfpos = xpos as f32 / h * 2.0 - 1.8;
    let yfpos = ypos as f32 / h * 2.0 - 1.0;
    for ycoord in hstart..hend {
        let y = ycoord as f32;
        for xcoord in (0..width).step_by(4) {
            let x = xcoord as f32;

            let ax = x86_64::_mm_set1_ps(xfpos);
            let ay = x86_64::_mm_set1_ps(yfpos);
            iterations = x86_64::_mm_set1_epi32(1);
            let mut zx = x86_64::_mm_set_ps(
                x / h * 2.0 - 1.8,
                (x + 1.0) / h * 2.0 - 1.8,
                (x + 2.0) / h * 2.0 - 1.8,
                (x + 3.0) / h * 2.0 - 1.8,
            );
            let mut zy = x86_64::_mm_set1_ps(y / h * 2.0 - 1.0);

            let iter_cmp = x86_64::_mm_set1_epi32(max_iterations as i32);
            let compare = x86_64::_mm_set1_ps(4.0);

            loop {
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
                        (value & 8) >> 3,
                        (value & 4) >> 2,
                        (value & 2) >> 1,
                        value & 1,
                    ),
                );
                let cmp_iter = x86_64::_mm_cmpeq_epi32(iterations, iter_cmp);
                let msk = x86_64::_mm_movemask_epi8(cmp_iter);
                if msk > 0 {
                    break;
                }
            }

            let mut iter: [i32; 4] = [0; 4];
            x86_64::_mm_storeu_epi32(iter.as_mut_ptr(), iterations);
            block[ycoord - hstart][xcoord] = iter[3] as IterationType;
            block[ycoord - hstart][xcoord + 1] = iter[2] as IterationType;
            block[ycoord - hstart][xcoord + 2] = iter[1] as IterationType;
            block[ycoord - hstart][xcoord + 3] = iter[0] as IterationType;
        }
        for xcoord in (width - rem)..width {
            let x = xcoord as f32;
            let mut z = (x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let a = (xfpos as f32, yfpos as f32);
            let mut iterations = 0;
            while iterations < max_iterations && z.0 * z.0 + z.1 * z.1 < 4.0 {
                z = (z.0 * z.0 - z.1 * z.1, 2.0 * z.0 * z.1);
                z = (z.0 + a.0, z.1 + a.1);
                iterations += 1;
            }
            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn juliaf32_simd256(
    hstart: usize,
    hend: usize,
    max_iterations: u32,
    pow: u32,
    width: usize,
    height: usize,
    block: &mut Vec<Vec<u32>>,
    xpos: i32,
    ypos: i32,
) -> Block {
    let h = height as f32;
    let mut iter: [i32; 8] = [0; 8];
    let size = width;
    let rem = size & 7;

    let xfpos = xpos as f32 / h * 2.0 - 1.8;
    let yfpos = ypos as f32 / h * 2.0 - 1.0;
    for ycoord in hstart..hend {
        let y = ycoord as f32;
        for xcoord in (0..(width - rem)).step_by(8) {
            let x = xcoord as f32;
            // print!("start {} ", ycoord);
            let ax = x86_64::_mm256_set1_ps(xfpos as f32);
            let ay = x86_64::_mm256_set1_ps(yfpos as f32);
            let mut iterations = x86_64::_mm256_set1_epi32(1);
            let mut zx = x86_64::_mm256_set_ps(
                x / h * 2.0 - 1.8,
                (x + 1.0) / h * 2.0 - 1.8,
                (x + 2.0) / h * 2.0 - 1.8,
                (x + 3.0) / h * 2.0 - 1.8,
                (x + 4.0) / h * 2.0 - 1.8,
                (x + 5.0) / h * 2.0 - 1.8,
                (x + 6.0) / h * 2.0 - 1.8,
                (x + 7.0) / h * 2.0 - 1.8,
            );
            let mut zy = x86_64::_mm256_set1_ps(y / h * 2.0 - 1.0);

            let iter_cmp = x86_64::_mm256_set1_epi32(max_iterations as i32);
            let compare = x86_64::_mm256_set1_ps(4.0);

            loop {
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
                        (value & 128) >> 7,
                        (value & 64) >> 6,
                        (value & 32) >> 5,
                        (value & 16) >> 4,
                        (value & 8) >> 3,
                        (value & 4) >> 2,
                        (value & 2) >> 1,
                        value & 1,
                    ),
                );

                let cmp_iter = x86_64::_mm256_cmpeq_epi32(iter_cmp, iterations);
                let msk = x86_64::_mm256_movemask_epi8(cmp_iter);
                if msk != 0 {
                    break;
                }
            }
            x86_64::_mm256_storeu_epi32(iter.as_mut_ptr(), iterations);
            // println!("{:?}", iter);
            block[ycoord - hstart][xcoord] = iter[7] as IterationType;
            block[ycoord - hstart][xcoord + 1] = iter[6] as IterationType;
            block[ycoord - hstart][xcoord + 2] = iter[5] as IterationType;
            block[ycoord - hstart][xcoord + 3] = iter[4] as IterationType;
            block[ycoord - hstart][xcoord + 4] = iter[3] as IterationType;
            block[ycoord - hstart][xcoord + 5] = iter[2] as IterationType;
            block[ycoord - hstart][xcoord + 6] = iter[1] as IterationType;
            block[ycoord - hstart][xcoord + 7] = iter[0] as IterationType;
        }
        for xcoord in (width - rem)..width {
            let x = xcoord as f32;
            let mut z = (x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let a = (xfpos as f32, yfpos as f32);
            let mut iterations = 0;
            while iterations < max_iterations && z.0 * z.0 + z.1 * z.1 < 4.0 {
                z = (z.0 * z.0 - z.1 * z.1, 2.0 * z.0 * z.1);
                z = (z.0 + a.0, z.1 + a.1);
                iterations += 1;
            }
            block[ycoord - hstart][xcoord] = iterations;
        }
    }
    (hstart, hend)
}