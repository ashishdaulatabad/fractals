use crate::complex::Complex;
use conv::prelude::*;
use core::fmt;
use itertools::Itertools;
use num_traits::pow::Pow;
use std::default::Default;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign},
};
use Vec;

pub trait PolynomialOperationTypes {}

impl PolynomialOperationTypes for u8 {}
impl PolynomialOperationTypes for u16 {}
impl PolynomialOperationTypes for u32 {}
impl PolynomialOperationTypes for usize {}
impl PolynomialOperationTypes for u64 {}
impl PolynomialOperationTypes for u128 {}
impl PolynomialOperationTypes for i8 {}
impl PolynomialOperationTypes for i16 {}
impl PolynomialOperationTypes for i32 {}
impl PolynomialOperationTypes for i64 {}
impl PolynomialOperationTypes for i128 {}
impl PolynomialOperationTypes for f32 {}
impl PolynomialOperationTypes for f64 {}
impl PolynomialOperationTypes for Complex {}

#[derive(Debug, Clone)]
pub struct Polynomial<T: PolynomialOperationTypes>
where
    T: Copy,
{
    pub poly: Vec<T>,
    pub deg: u32,
}

impl<'b, T: PolynomialOperationTypes> Polynomial<T>
where
    T: Mul<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Div<Output = T>
        + Pow<T, Output = T>
        + ValueFrom<u32>
        + Copy,
{
    pub fn new() -> Self {
        Self {
            poly: Vec::<T>::new(),
            deg: 0_u32,
        }
    }

    pub fn p_add(self, other: &'b Polynomial<T>) -> Polynomial<T> {
        Polynomial {
            poly: self
                .poly
                .iter()
                .zip_longest(other.poly.iter())
                .map(|c| match c {
                    itertools::EitherOrBoth::Both(l, r) => *l + *r,
                    itertools::EitherOrBoth::Left(l) => *l,
                    itertools::EitherOrBoth::Right(r) => *r,
                })
                .collect::<Vec<T>>(),
            deg: if self.deg > other.deg {
                self.deg
            } else {
                other.deg
            },
        }
    }

    pub fn evaluate(&self, _x: T) -> T
    where
        T: ValueFrom<usize>,
    {
        self.poly
            .iter()
            .enumerate()
            .map(|(index, item)| -> T { *item * _x.pow(index.value_into().unwrap()) })
            .reduce(|accumulator, val| accumulator + val)
            .unwrap()
    }

    pub fn derivative(&self) -> Self {
        Self {
            poly: (1..self.poly.len())
                .map(|c| -> T { T::value_from(c as u32).unwrap() * self.poly[c] })
                .collect::<Vec<T>>(),
            deg: self.deg - 1,
        }
    }
}

impl<T: fmt::Display + Copy> Display for Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_output = self
            .poly
            .iter()
            .rev()
            .enumerate()
            .map(|(index, val)| -> String {
                format!("{}x^({})", val.to_string(), self.poly.len() - index - 1)
            })
            .collect::<Vec<String>>()
            .join(" + ");
        f.write_str(format!("Poly({})", str_output).as_str())
    }
}

impl<'a, 'b, T: Copy + Add<Output = T>> Add<&'b Polynomial<T>> for &'a Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    type Output = Polynomial<T>;
    fn add(self, other: &'b Polynomial<T>) -> Polynomial<T> {
        Polynomial {
            poly: self
                .poly
                .iter()
                .zip_longest(other.poly.iter())
                .map(|c| -> T {
                    match c {
                        itertools::EitherOrBoth::Both(l, r) => *l + *r,
                        itertools::EitherOrBoth::Left(l) => *l,
                        itertools::EitherOrBoth::Right(r) => *r,
                    }
                })
                .collect::<Vec<T>>(),
            deg: if self.deg > other.deg {
                self.deg
            } else {
                other.deg
            },
        }
    }
}

impl<'b, T: Copy + AddAssign> AddAssign<&'b Polynomial<T>> for Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    fn add_assign(&mut self, rhs: &'b Polynomial<T>) {
        if rhs.poly.len() > self.poly.len() {
            self.poly.extend(rhs.poly.iter().skip(self.poly.len()));
        }
        if rhs.poly.len() < self.poly.len() {
            self.poly[..rhs.poly.len()]
                .iter_mut()
                .enumerate()
                .for_each(|(index, elem)| *elem += rhs.poly[index]);
        }
    }
}

impl<'b, T: Copy + Mul<Output = T> + AddAssign + Default + PartialEq> Mul<&'b Polynomial<T>>
    for Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    type Output = Polynomial<T>;

    fn mul(self, rhs: &'b Polynomial<T>) -> Self::Output {
        let mut new_poly: Polynomial<T> = Polynomial {
            poly: vec![Default::default(); self.deg as usize + rhs.deg as usize + 1],
            deg: self.poly.len() as u32 + rhs.poly.len() as u32 - 2_u32,
        };

        for first_index in 0..self.poly.len() {
            if self.poly[first_index] != Default::default() {
                for second_index in 0..rhs.poly.len() {
                    new_poly.poly[first_index + second_index] +=
                        self.poly[first_index] * rhs.poly[second_index];
                }
            }
        }

        new_poly
    }
}

impl<'b, T: Copy + Mul<Output = T> + AddAssign + Default + PartialEq> MulAssign<&'b Polynomial<T>>
    for Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    fn mul_assign(&mut self, rhs: &'b Polynomial<T>) {
        let mut new_poly: Vec<T> =
            vec![Default::default(); self.deg as usize + rhs.deg as usize + 1];

        for first_index in 0..self.poly.len() {
            if self.poly[first_index] != Default::default() {
                for second_index in 0..rhs.poly.len() {
                    new_poly[first_index + second_index] +=
                        self.poly[first_index] * rhs.poly[second_index];
                }
            }
        }

        *self = Polynomial {
            poly: new_poly,
            deg: (self.poly.len() + rhs.poly.len() - 1) as u32,
        }
    }
}

impl<'b, T: Copy + SubAssign + Neg<Output = T>> SubAssign<&'b Polynomial<T>> for Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    fn sub_assign(&mut self, rhs: &'b Polynomial<T>) {
        if rhs.poly.len() > self.poly.len() {
            self.poly.extend(
                rhs.poly
                    .iter()
                    .skip(self.poly.len())
                    .map(|item| -> T { -*item }),
            );
        }
        if rhs.poly.len() < self.poly.len() {
            self.poly[..rhs.poly.len()]
                .iter_mut()
                .enumerate()
                .for_each(|(index, elem)| *elem -= rhs.poly[index]);
        }
    }
}

impl<'b, T: Copy + Sub<Output = T> + Neg<Output = T>> Sub<&'b Polynomial<T>> for Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    type Output = Polynomial<T>;
    fn sub(self, other: &'b Polynomial<T>) -> Polynomial<T> {
        Polynomial {
            poly: self
                .poly
                .iter()
                .zip_longest(other.poly.iter())
                .map(|c| match c {
                    itertools::EitherOrBoth::Both(l, r) => *l - *r,
                    itertools::EitherOrBoth::Left(l) => *l,
                    itertools::EitherOrBoth::Right(r) => -(*r),
                })
                .collect::<Vec<T>>(),
            deg: if self.deg > other.deg {
                self.deg
            } else {
                other.deg
            },
        }
    }
}

impl<T: Copy + Sub<Output = T> + Neg<Output = T>> Sub for Polynomial<T>
where
    T: PolynomialOperationTypes,
{
    type Output = Polynomial<T>;
    fn sub(self, other: Polynomial<T>) -> Polynomial<T> {
        Polynomial {
            poly: self
                .poly
                .iter()
                .zip_longest(other.poly.iter())
                .map(|c| match c {
                    itertools::EitherOrBoth::Both(l, r) => *l - *r,
                    itertools::EitherOrBoth::Left(l) => *l,
                    itertools::EitherOrBoth::Right(r) => -(*r),
                })
                .collect::<Vec<T>>(),
            deg: if self.deg > other.deg {
                self.deg
            } else {
                other.deg
            },
        }
    }
}
