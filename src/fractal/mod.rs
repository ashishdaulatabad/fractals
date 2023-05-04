use pixel_canvas::Color;
use std::thread::ScopedJoinHandle;
mod burning_ship;
mod julia;
mod mandelbrot;
mod newton;

type IterationType = u32;
type FSignature =
    unsafe fn(usize, usize, u32, u32, usize, usize, &mut Vec<Vec<u32>>, i32, i32) -> Block;
type Block = (usize, usize);

/// Enum for Fractal Type
#[derive(Clone, Debug, Copy)]
pub enum FractalType {
    Mandelbrot,
    Julia,
    BurningShip,
    Newton,
}

#[derive(Clone, Copy, Debug)]
pub enum Precision {
    F32,
    F64,
}

#[derive(Clone, Copy, Debug)]
pub enum InstructionSet {
    None,
    SSE,
    AVX,
}

pub struct Fractal {
    fractal_type: FractalType,
    num_threads: Option<u8>,
    max_iter: u16,
    color_buffer: Vec<Color>,
    width: u16,
    height: u16,
    pow: u32,
    iset: InstructionSet,
    precision: Precision,
    th_block: Vec<Vec<Vec<u32>>>,
}

impl Fractal {
    pub fn new() -> Self {
        Fractal {
            fractal_type: FractalType::Mandelbrot,
            num_threads: None,
            max_iter: 100,
            width: 1280,
            pow: 2,
            color_buffer: crate::utils::color::build_color_array(100),
            height: 720,
            iset: InstructionSet::None,
            precision: Precision::F32,
            th_block: vec![vec![vec![0; 1280]; 720]; 1],
        }
    }

    pub fn set_iset(mut self, iset: InstructionSet) -> Self {
        self.iset = iset;
        self
    }

    pub fn set_pow(mut self, pow: u32) -> Self {
        self.pow = pow;
        self
    }

    pub fn set_prec(mut self, prec: Precision) -> Self {
        self.precision = prec;
        self
    }

    pub fn set_max_iter(mut self, iter: u16) -> Self {
        self.max_iter = iter;
        self.color_buffer = crate::utils::color::build_color_array(iter as u32);
        self
    }

    pub fn set_num_threads(mut self, threads: u8) -> Self {
        self.num_threads = Some(threads);
        let th_height = self.height / threads as u16;
        let th_rem = self.height % threads as u16;
        self.th_block = Vec::new();
        for _ in 0..threads - 1 {
            self.th_block
                .push(vec![vec![0; self.width as usize]; th_height as usize]);
        }
        self.th_block.push(vec![
            vec![0; self.width as usize];
            th_height as usize + th_rem as usize
        ]);
        self
    }

    pub fn get_dim(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn set_window_dim(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        match self.num_threads {
            None => self.th_block = vec![vec![vec![0; width as usize]; height as usize]; 1],
            Some(threads) => {
                let th_height = self.height / threads as u16;
                let th_rem = self.height % threads as u16;
                for _ in 0..threads - 1 {
                    self.th_block
                        .push(vec![vec![0; self.width as usize]; th_height as usize]);
                }
                self.th_block.push(vec![
                    vec![0; self.width as usize];
                    th_height as usize + th_rem as usize
                ]);
            }
        };
        self
    }

    pub fn set_fractal(mut self, ftype: FractalType) -> Self {
        self.fractal_type = ftype;
        self
    }

    pub fn draw(&mut self, image: &mut pixel_canvas::Image, xpos: i32, ypos: i32) {
        match self.num_threads {
            None => self.draw_st(image, xpos, ypos),
            Some(th) => self.draw_mt(image, th, xpos, ypos),
        }
    }

    pub fn draw_st(&mut self, image: &mut pixel_canvas::Image, xpos: i32, ypos: i32) {
        let perform_op = match self.fractal_type {
            FractalType::BurningShip => burning_ship::fn_(self.precision, self.iset),
            FractalType::Julia => julia::fn_(self.precision, self.iset),
            _ => mandelbrot::fn_(self.precision, self.iset),
        };
        let (_, _) = unsafe {
            perform_op(
                0,
                self.height as usize,
                self.max_iter as u32,
                self.pow,
                self.width as usize,
                self.height as usize,
                &mut self.th_block[0],
                xpos,
                ypos,
            )
        };
        for col in 0..(self.height as usize) {
            for row in 0..self.width {
                let idx = self.th_block[0][col][row as usize];
                image[pixel_canvas::RC(col as usize, row as usize)] =
                    self.color_buffer[idx as usize];
            }
        }
    }

    pub fn draw_mt(&mut self, image: &mut pixel_canvas::Image, thread: u8, xpos: i32, ypos: i32) {
        draw_mt(
            image,
            &self.color_buffer,
            self.fractal_type,
            self.width,
            self.height,
            self.max_iter,
            self.precision,
            self.iset,
            self.pow,
            thread,
            &mut self.th_block,
            xpos,
            ypos,
        );
    }
}

fn draw_mt(
    image: &mut pixel_canvas::Image,
    color: &Vec<Color>,
    ftype: FractalType,
    width: u16,
    height: u16,
    max_iter: u16,
    prec: Precision,
    iset: InstructionSet,
    pow: u32,
    thread: u8,
    blocks: &mut Vec<Vec<Vec<u32>>>,
    xpos: i32,
    ypos: i32,
) {
    let sthread_height: usize = (height / (thread as u16)) as usize;

    let perform_op = match ftype {
        FractalType::BurningShip => burning_ship::fn_(prec, iset),
        FractalType::Julia => julia::fn_(prec, iset),
        FractalType::Newton => newton::newton,
        _ => mandelbrot::fn_(prec, iset),
    };
    unsafe {
        std::thread::scope(|scope| {
            let _ = blocks
                .iter_mut()
                .enumerate()
                .map(|(i, mut c)| {
                    scope.spawn(move || {
                        perform_op(
                            sthread_height * i,
                            sthread_height * (i + 1),
                            max_iter as u32,
                            pow,
                            width as usize,
                            height as usize,
                            &mut c,
                            xpos,
                            ypos,
                        )
                    })
                })
                .collect::<Vec<ScopedJoinHandle<(usize, usize)>>>();
        });
    }
    for (i, block) in blocks.iter().enumerate() {
        for (col, val) in block.iter().enumerate() {
            for (row, elem) in val.iter().enumerate() {
                let idx = blocks[i][col as usize][row] as usize;
                image[pixel_canvas::RC(sthread_height * i + col as usize, row)] = color[idx];
            }
        }
    }
}
