// use crate::color;
// use crate::complex;
// use pixel_canvas::RC;
// use std::thread::JoinHandle;

// fn parallel_buddhabrot(
//     hstart: usize,
//     hend: usize,
//     max_iterations: u32,
//     pow: u32,
//     is_julia: bool,
//     width: usize,
//     height: usize,
//     block: &mut Vec<Vec<u32>>, xpos: i32, ypos: i32 ,
// ) -> (Vec<Vec<usize>>, usize, usize) {
//     let (mut c, mut z): (complex::Complex, complex::Complex);
//     let h = height as f64;
//     let (mut xfloat, mut yfloat): (f64, f64);
//     let mut iterations;
//     // let mut image = img.unwrap();

//     for x in hstart..hend {
//         xfloat = x as f64;
//         for y in 0..width {
//             yfloat = y as f64;
//             if !is_julia {
//                 c = complex::Complex::new(xfloat / h * 2.0 - 2.0, yfloat / h * 2.0 - 1.0);
//                 z = complex::Complex::new(0.0, 0.0);
//             } else {
//                 z = complex::Complex::new(xfloat / h * 2.0 - 2.0, yfloat / h * 2.0 - 1.0);
//                 c = complex::Complex::new(-0.74543, 0.11301);
//             }
//             iterations = 0;
//             while iterations < max_iterations && z.abs_sq() < 4.0 {
//                 z *= z;
//                 z += c;

//                 iterations += 1;
//             }

//             block[y][x - hstart] = iterations as usize;
//         }
//     }
//     (hstart, hend)
// }

// pub fn mandelbrot(image: &mut pixel_canvas::Image, is_julia: bool, power: u32) {
//     let thread_num = 16;
//     let total_iterations = 150 as u32;
//     let (width, width): (usize, usize) = (image.width(), image.width());
//     let color = color::build_color_array(total_iterations);
//     let sthread_width: usize = width / thread_num;

//     let mut thread_handles: Vec<JoinHandle<(Vec<Vec<usize>>, usize, usize)>> = (0..thread_num - 1)
//         .into_iter()
//         .map(|c| {
//             std::thread::spawn(move || {
//                 parallel_mandelbrot(
//                     sthread_width * c,
//                     sthread_width * (c + 1),
//                     total_iterations,
//                     power,
//                     is_julia,
//                     width,
//                 )
//             })
//         })
//         .collect::<Vec<JoinHandle<(Vec<Vec<usize>>, usize, usize)>>>();

//     thread_handles.push(std::thread::spawn(move || {
//         parallel_mandelbrot(
//             sthread_width * (thread_num - 1),
//             width,
//             total_iterations,
//             power,
//             is_julia,
//             width,
//         )
//     }));

//     let results = thread_handles
//         .into_iter()
//         .map(|c| c.join().unwrap())
//         .collect::<Vec<(Vec<Vec<usize>>, usize, usize)>>();

//     for (vec, start, end) in results {
//         for row in start..end {
//             for col in 0..width {
//                 block[RC(col, row)] = color[vec[col][row - start]];
//             }
//         }
//     }
//     println!("Done");
// }
