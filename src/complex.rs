use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// Complex number for computing.
#[derive(Debug, Copy, Clone)]
pub struct Complex {
    pub real: f64,
    pub img: f64,
}

impl Add<&Complex> for Complex {
    type Output = Self;
    fn add(self, b: &Complex) -> Complex {
        Self {
            real: self.real + b.real,
            img: self.img + b.img,
        }
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Complex {
        Self {
            real: -self.real,
            img: -self.img,
        }
    }
}

impl AddAssign<&Complex> for Complex {
    fn add_assign(&mut self, b: &Self) {
        self.real += b.real;
        self.img += b.img;
    }
}

impl AddAssign<Complex> for Complex {
    fn add_assign(&mut self, b: Self) {
        self.real += b.real;
        self.img += b.img;
    }
}

impl Sub<&Complex> for Complex {
    type Output = Self;

    fn sub(self, b: &Complex) -> Complex {
        Self {
            real: self.real - b.real,
            img: self.img - b.img,
        }
    }
}

impl SubAssign<&Complex> for Complex {
    fn sub_assign(&mut self, b: &Self) {
        self.real -= b.real;
        self.img -= b.img;
    }
}

impl Mul<&Complex> for Complex {
    type Output = Self;

    fn mul(self, b: &Complex) -> Complex {
        Self {
            real: self.real * b.real - self.img * b.img,
            img: self.img * b.real + self.real * b.img,
        }
    }
}

impl Mul<Complex> for Complex {
    type Output = Self;

    fn mul(self, b: Complex) -> Complex {
        Self {
            real: self.real * b.real - self.img * b.img,
            img: self.img * b.real + self.real * b.img,
        }
    }
}

impl MulAssign<&Complex> for Complex {
    fn mul_assign(&mut self, b: &Self) {
        *self = Self {
            real: self.real * b.real - self.img * b.img,
            img: self.img * b.real + self.real * b.img,
        }
    }
}

impl MulAssign<Complex> for Complex {
    fn mul_assign(&mut self, b: Self) {
        *self = Self {
            real: self.real * b.real - self.img * b.img,
            img: self.img * b.real + self.real * b.img,
        }
    }
}

impl Div<&Complex> for Complex {
    type Output = Self;

    fn div(self, b: &Complex) -> Complex {
        let abs = b.real * b.real + b.img * b.img;
        Self {
            real: (self.real * b.real + self.img * b.img) / abs,
            img: (self.img * b.real - self.real * b.img) / abs,
        }
    }
}

impl Div<Complex> for Complex {
    type Output = Self;

    fn div(self, b: Complex) -> Complex {
        let abs = b.real * b.real + b.img * b.img;
        Self {
            real: (self.real * b.real + self.img * b.img) / abs,
            img: (self.img * b.real - self.real * b.img) / abs,
        }
    }
}

impl DivAssign<Complex> for Complex {
    fn div_assign(&mut self, b: Self) {
        let abs = b.real * b.real + b.img * b.img;
        *self = Self {
            real: (self.real * b.real + self.img * b.img) / abs,
            img: (self.img * b.real - self.real * b.img) / abs,
        }
    }
}

impl DivAssign<&Complex> for Complex {
    fn div_assign(&mut self, b: &Self) {
        let abs = b.real * b.real + b.img * b.img;
        *self = Self {
            real: (self.real * b.real + self.img * b.img) / abs,
            img: (self.img * b.real - self.real * b.img) / abs,
        }
    }
}

impl Complex {
    pub fn new(r: f64, i: f64) -> Self {
        Self { real: r, img: i }
    }

    pub fn abs_sq(self) -> f64 {
        self.real * self.real + self.img * self.img
    }

    pub fn ipow(self, mut power: u32) -> Self {
        match power {
            0 => Self {
                real: 1.0,
                img: 0.0,
            },
            1 => self,
            2 => self * self,
            3 => self * self * self,
            _ => {
                let mut result: Complex = Self {
                    real: 1.0,
                    img: 0.0,
                };
                let mut mul = self;
                while power > 0 {
                    if power & 1 == 1 {
                        result *= mul;
                    }
                    mul *= mul;
                    power >>= 1;
                }
                result
            }
        }
    }

    pub fn conjugate(self) -> Self {
        Self {
            real: self.real,
            img: -self.img,
        }
    }
}
