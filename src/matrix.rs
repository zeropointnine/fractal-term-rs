// Wrapper for a 2D matrix of values used for the Mandelbrot map data

// TODO: Consider trying 1-dimensional structure
//       This might make it more viable to do mem copy type operations, which could interesting to try...

use std::fmt;


#[derive(Clone)]
pub struct Matrix<T> {
	vec: Vec<Vec<T>>  
}


impl<T:Clone + Default> Matrix<T> {

	pub fn new(width: usize, height: usize) -> Matrix<T> {

		assert!(width > 0 && height > 0);

		// TODO: can a single macro statement init a '2d' vector?
		let mut vec: Vec<Vec<T>> = Vec::new();		
		for _ in 0..height {
			let v = vec!(T::default(); width);
			vec.push(v);
		}

		Matrix { vec: vec }
	}

	// rem, the first dimension is the row index (y), the second dimension is the column index (x)
	pub fn vec(&mut self) -> &mut Vec<Vec<T>> {
		&mut self.vec
	}
	
	pub fn width(&self) -> usize {
		self.vec[0].len()
	}
	pub fn height(&self) -> usize {
		self.vec.len()
	}
	pub fn get(&self, x: usize, y: usize) -> T {
		let row = &self.vec[y];
		row[x].clone()
	}
	pub fn set(&mut self, x: usize, y: usize, value:T) {
		self.vec[y][x] = value;
	}
	
	pub fn get_row(&self, y:usize) -> &Vec<T> {
		&self.vec[y]
	}
	
	/**
	 * Writes the full contents of 'src' into self starting at index 'start'
	 */
	pub fn copy_from(&mut self, src: &Matrix<T>, start: usize) {
		for src_y in 0..src.height() {
			let self_y = start + src_y;
			for x in 0..src.width() {
				self.set(x, self_y, src.get(x, src_y).clone());
			}
		}
	}
}


impl<T: fmt::Display> fmt::Debug for Matrix<T>  {

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

