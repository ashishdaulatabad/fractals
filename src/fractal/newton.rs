use crate::complex::Complex;
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
    let h = height as f64;
    let u = Complex::new((xpos as f64) / h * 2.0 - 1.8, (ypos as f64) / h * 2.0 - 1.0);
    let root = [
        Complex::new(1.0, 0.0),
        Complex::new(-0.5, 3.0_f64.sqrt() / 2.0),
        Complex::new(-0.5, -3.0_f64.sqrt() / 2.0),
    ];
    let a = Complex::new(-0.5, 0.0);

    let tol = 1e-9;
    for ycoord in hstart..hend {
        let y = ycoord as f64;
        for xcoord in 0..width {
            let x = xcoord as f64;
            let mut z = Complex::new(x / h * 2.0 - 1.8, y / h * 2.0 - 1.0);
            let mut iterations = 0;
            while iterations < max_iterations && z.abs_sq() < 4.0 {
                z = z - &(((z.ipow(pow) - &u) / z.ipow(pow - 1) * Complex::new(pow as f64, 0.0))
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
