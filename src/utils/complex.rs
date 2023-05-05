use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! impl_complex_for {
    ($class:ident, $type:ident) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $class {
            pub real: $type,
            pub img: $type,
        }

        impl Add<&$class> for $class {
            type Output = Self;
            #[inline(always)]
            fn add(self, b: &$class) -> $class {
                Self {
                    real: self.real + b.real,
                    img: self.img + b.img,
                }
            }
        }

        impl Neg for $class {
            type Output = Self;
            #[inline(always)]
            fn neg(self) -> $class {
                Self {
                    real: -self.real,
                    img: -self.img,
                }
            }
        }

        impl AddAssign<&$class> for $class {
            #[inline(always)]
            fn add_assign(&mut self, b: &Self) {
                self.real += b.real;
                self.img += b.img;
            }
        }

        impl AddAssign<$class> for $class {
            #[inline(always)]
            fn add_assign(&mut self, b: Self) {
                self.real += b.real;
                self.img += b.img;
            }
        }

        impl Sub<&$class> for $class {
            type Output = Self;

            #[inline(always)]
            fn sub(self, b: &$class) -> $class {
                Self {
                    real: self.real - b.real,
                    img: self.img - b.img,
                }
            }
        }

        impl SubAssign<&$class> for $class {
            #[inline(always)]
            fn sub_assign(&mut self, b: &Self) {
                self.real -= b.real;
                self.img -= b.img;
            }
        }

        impl Mul<&$class> for $class {
            type Output = Self;

            #[inline(always)]
            fn mul(self, b: &$class) -> $class {
                Self {
                    real: self.real * b.real - self.img * b.img,
                    img: self.img * b.real + self.real * b.img,
                }
            }
        }

        impl Mul<$class> for $class {
            type Output = Self;

            #[inline(always)]
            fn mul(self, b: $class) -> $class {
                Self {
                    real: self.real * b.real - self.img * b.img,
                    img: self.img * b.real + self.real * b.img,
                }
            }
        }

        impl MulAssign<&$class> for $class {
            #[inline(always)]
            fn mul_assign(&mut self, b: &Self) {
                *self = Self {
                    real: self.real * b.real - self.img * b.img,
                    img: self.img * b.real + self.real * b.img,
                }
            }
        }

        impl MulAssign<$class> for $class {
            #[inline(always)]
            fn mul_assign(&mut self, b: Self) {
                *self = Self {
                    real: self.real * b.real - self.img * b.img,
                    img: self.img * b.real + self.real * b.img,
                }
            }
        }

        impl Div<&$class> for $class {
            type Output = Self;

            fn div(self, b: &$class) -> $class {
                let abs = b.real * b.real + b.img * b.img;
                Self {
                    real: (self.real * b.real + self.img * b.img) / abs,
                    img: (self.img * b.real - self.real * b.img) / abs,
                }
            }
        }

        impl Div<$class> for $class {
            type Output = Self;

            fn div(self, b: $class) -> $class {
                let abs = b.real * b.real + b.img * b.img;
                Self {
                    real: (self.real * b.real + self.img * b.img) / abs,
                    img: (self.img * b.real - self.real * b.img) / abs,
                }
            }
        }

        impl DivAssign<$class> for $class {
            fn div_assign(&mut self, b: Self) {
                let abs = b.real * b.real + b.img * b.img;
                *self = Self {
                    real: (self.real * b.real + self.img * b.img) / abs,
                    img: (self.img * b.real - self.real * b.img) / abs,
                }
            }
        }

        impl DivAssign<&$class> for $class {
            fn div_assign(&mut self, b: &Self) {
                let abs = b.real * b.real + b.img * b.img;
                *self = Self {
                    real: (self.real * b.real + self.img * b.img) / abs,
                    img: (self.img * b.real - self.real * b.img) / abs,
                }
            }
        }

        impl $class {
            #[inline(always)]
            pub fn new(r: $type, i: $type) -> Self {
                Self { real: r, img: i }
            }

            #[inline(always)]
            pub fn abs_sq(self) -> $type {
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
                        let mut result: $class = Self {
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
    };
}
// $class number for computing.

impl_complex_for!(Complex64, f64);
impl_complex_for!(Complex32, f32);
