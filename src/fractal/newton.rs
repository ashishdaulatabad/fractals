use crate::utils::complex::{Complex32, Complex64};
use core::arch::x86_64;

use crate::fractal::{Block, IterationType};

use super::{FSignature, InstructionSet, Precision};

pub fn newton(
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
    let u = Complex32::new((xpos as f32) / h * 2.0 - 1.8, (ypos as f32) / h * 2.0 - 1.0);
    let hpow = 3;
    let root = [
        Complex32::new(1.0, 0.0),
        Complex32::new(-0.5, 3.0_f32.sqrt() / 2.0),
        Complex32::new(-0.5, -3.0_f32.sqrt() / 2.0),
    ];
    let a = Complex32::new(-0.03125, 0.0);

    let tol = 1e-6;
    for ycoord in hstart..hend {
        let y = ycoord as f32;
        for xcoord in 0..width {
            let x = xcoord as f32;
            let mut z = Complex32::new(x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let mut iterations = 0;
            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z = z - &(((z.ipow(hpow) - &u) / z.ipow(hpow - 1)
                    * Complex32::new(hpow as f32, 0.0))
                    * a);
                iterations += 1;

                for cr in root {
                    let diff = z - &cr;
                    if diff.real < tol && diff.img < tol {
                        break;
                    }
                }
            }
            block[ycoord - hstart][xcoord] = iterations as IterationType;
        }
    }
    (hstart, hend)
}
