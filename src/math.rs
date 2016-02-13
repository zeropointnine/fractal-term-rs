extern crate num;
use self::num::traits::Float;

pub fn normalize<T: Float>(val: T, min: T, max: T) -> T {
	(val - min) / (max - min)
}

pub fn interpolate<T:Float>(normed_val:T, min:T, max:T) -> T {
	min  +  (max - min) * normed_val
}

pub fn map<T:Float>(val:T, min1:T, max1:T, min2:T, max2:T) -> T {
	interpolate( normalize(val, min1, max1), min2, max2)
}