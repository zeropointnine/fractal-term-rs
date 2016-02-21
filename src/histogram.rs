use std::ops::Range;
use matrix::Matrix;


pub struct Histogram {
	counts: Vec<u16>,	// each element is a 'bin' which gets incremented
}

impl Histogram {
	
	pub fn new(max_val: usize) -> Histogram {
		let counts = vec!(u16::default(); max_val + 1); 
		Histogram { counts: counts }
		
	}
	
	pub fn get_range(&mut self, matrix: &Matrix<u16>, lower_thresh_ratio: f64, upper_thresh_ratio: f64) -> (usize, usize) {

		self.count(&matrix);

		let sum_thresh =  (matrix.width() as f64 * matrix.height() as f64) * lower_thresh_ratio;
		let mut lower_index = 0;
		let mut sum = 0;
		for i in 0..self.counts.len() {
			sum += self.counts[i];
			if sum as f64 > sum_thresh {
				lower_index =  if i == 0 { 0 as usize } else { i - 1 };  // rewind by 1
				break;
			}
		} 

		let sum_thresh =  (matrix.width() as f64 * matrix.height() as f64) * upper_thresh_ratio;
		let mut upper_index = 0;		
		let mut sum = 0;
		for i in (0..self.counts.len()).rev() {
			sum += self.counts[i];
			if sum as f64 > sum_thresh {
				upper_index =  if i == self.counts.len() - 1 { self.counts.len() - 1 } else { i - 1 };
				break;
			}
		} 
		
		(lower_index, upper_index)
	}
	
	fn count(&mut self, matrix: &Matrix<u16>) {

		for i in 0..self.counts.len() {
			self.counts[i] = 0;
		}

		for y in 0..matrix.height() {
			for x in 0..matrix.width() {
				let i = matrix.get(x, y) as usize;
				self.counts[i] += 1; 
			}
		}
	}
}
