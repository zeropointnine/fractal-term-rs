// Don't use.

// Complex number
// As an exercise in learning generics, I added the 'num' crate to enable making the data type 'generic' (ie, f32 or f64)
// But because the 'num' create actually has a Complex data type, there is no need for this at all! :)


extern crate num;
use self::num::traits::Float;
use std::ops::{Add, Mul};

#[derive(Debug)]
pub struct Complex<T:Float> {
    pub real: T,
    pub imag: T,
}

impl<T:Float> Complex<T> {
    fn len(&self) {
        (self.real * self.real + self.imag * self.imag).sqrt();
    }
}

impl<T:Float> Add for Complex<T> {
    type Output = Complex<T>;
    fn add(self, other: Complex<T>) -> Self::Output {
        Complex {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }
}

impl<T:Float> Mul for Complex<T> {
    type Output = Complex<T>;
    fn mul(self, other: Complex<T>) -> Self::Output {
        Complex {
            real: self.real * other.real - self.imag * other.imag,
            imag: self.imag * other.real + self.real * other.imag,
        }
    }
}
