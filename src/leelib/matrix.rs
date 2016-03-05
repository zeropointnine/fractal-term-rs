use std::fmt;


/**
 * Wrapper for a 2D matrix of values
 */
#[derive(Clone)]
pub struct Matrix<T> {
	vec: Vec<Vec<T>>,
	index:usize,  
}


impl<T:Clone + Default> Matrix<T> {

	pub fn new(width: usize, height: usize) -> Matrix<T> {

		assert!(width > 0 && height > 0, format!("width and height must be > 0:  {} {}", width, height));

		let mut vec: Vec<Vec<T>> = Vec::new();		
		for _ in 0..height {
			let v = vec!(T::default(); width);
			vec.push(v);
		}

		Matrix { vec: vec, index:0 }
	}

	// rem, the first dimension is the row index (y), 
	// the second dimension is the column index (x)
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
	pub fn get_ref(&mut self, x: usize, y: usize) -> &mut T {  // test
		let row = &mut self.vec[y];
		&mut row[x]
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
	pub fn copy_from(&mut self, src: &Matrix<T>, start_y: usize) {
		for src_y in 0..src.height() {
			let self_y = start_y + src_y;
			for x in 0..src.width() {
				self.set(x, self_y, src.get(x, src_y).clone());
			}
		}
	}
}

impl<T: fmt::Display> fmt::Debug for Matrix<T>  {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    	let mut string = String::new();
    	for row in self.vec.iter() {
			let mut s = String::new();
    		for el in row.iter() {
    			s = s + &format!("{:>4}", el);
    		}
			string = string + &s + "\n";
    	}
    	write!(f, "{}", string)
    }
}

impl Matrix<u8> {  

	 // TODO: want to use num::integer::Integer, but that won't cast to f64 (?)

	/** 
	 * Interpolates between `m1` and `m2` using `ratio`, writing the result into `dest`
	 */ 
	pub fn interpolate(ratio: f64, m1: &Matrix<u8>, m2: &Matrix<u8>, dest: &mut Matrix<u8>) {

		assert!(m1.width() == m2.width() && m2.width() == dest.width() && 
				m1.height() == m2.height() && m2.height() == dest.height(), 
				"Matrices must have same size");

		for y in 0..m1.height() {
			for x in 0..m1.width() {
				 let r1 = m1.get(x, y);
				 let r2 = m2.get(x, y);
				 let r3 = r1 as f64  +  (r2 as f64 - r1 as f64) * ratio;
				 dest.set(x, y, r3 as u8);  
			}
		}
	}

	/** 
	 * Additionally uses max value info
	 */ 
	pub fn interpolate2(ratio: f64, m1: &Matrix<u8>, max1: u8, m2: &Matrix<u8>, max2: u8, dest: &mut Matrix<u8>) {

		assert!(m1.width() == m2.width() && m2.width() == dest.width() && 
				m1.height() == m2.height() && m2.height() == dest.height(), 
				"Matrices must have same size");

		for y in 0..m1.height() {
			for x in 0..m1.width() {
				 let r1 = m1.get(x, y) as f64 / max1 as f64;
				 let r2 = m2.get(x, y) as f64 / max2 as f64;
				 let r3 = r1 + (r2 - r1) * ratio;
				 let res = (r3 * max2 as f64) as u8;
				 dest.set(x, y, res);  
			}
		}
	}
}


// IntoIterator

pub struct MatrixIntoIterator<'a, T: 'a> {
    matrix: &'a Matrix<T>,
    index: usize,
}

impl<'a, T: Clone + Default> IntoIterator for &'a Matrix<T> {
    type Item = T;
    type IntoIter = MatrixIntoIterator<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        MatrixIntoIterator { matrix: self, index: 0 }
    }
}

impl<'a, T: Clone + Default> Iterator for MatrixIntoIterator<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let y = self.index / self.matrix.width();
        let x = self.index % self.matrix.width();
        self.index += 1;
        if y >= self.matrix.height() {
        	None
        } else {
			Some(self.matrix.get(x, y))
        }
    }
}
