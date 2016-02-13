// Contains a 2-dimensional matrix of numbers used for the Mandelbrot map data

// TODO: Consider trying 1-dimensional structure
//       This might make it more viable to do mem copy type operations, which could interesting to try...

use std::fmt;


pub struct NumMap<T> {
	vec: Vec<Vec<T>>  
}


impl<T:Clone + Default> NumMap<T> {

	pub fn new(width: usize, height: usize) -> NumMap<T> {

		assert!(width > 0 && height > 0);

		// TODO: can a single macro statement init a '2d' vector?
		let mut vec: Vec<Vec<T>> = Vec::new();		
		for _ in 0..height {
			let v = vec!(T::default(); width);
			vec.push(v);
		}

		NumMap { vec: vec }
	}

	// rem, the first dimension is the row index (y), the second dimension is the column index (x)
	pub fn vec(&mut self) -> &mut Vec<Vec<T>> {
		&mut self.vec
	}
}


impl<T: fmt::Display> fmt::Debug for NumMap<T>  {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

    	let mut string = String::new();
		let cols = self.vec[0].len();
		for row in 0..self.vec.len() {
			let mut s = String::new();
			for col in 0..cols {
				let el = format!("{:>4}", self.vec[row][col]);  // 4 characters wide, right-aligned
				s = s + &el;
			}
			string = string + &s + "\n";
		}

    	write!(f, "{}", string)
    }
}
