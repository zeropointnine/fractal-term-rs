use matrix::Matrix;
use math;


/**
 * Experiment
 */ 
pub struct Histogram {
	counts: Vec<u16>  	
}

impl Histogram {
	
	pub fn new(max_val: usize) -> Histogram {
		let counts = vec!(u16::default(); max_val + 1);
		let sub = vec!(u16::default(); max_val + 1); 
		Histogram { counts: counts }
	}
	
	/**
	 * lower/upper_thresh_ratio:
	 * the ratio of the amount of upper and lower values to discard when calculating the range
	 * 
	 * returns the range where values occur, and the 'center of gravity' ratio (-1 to +1) within that range  
	 */ 
	pub fn calc(&mut self, matrix: &Matrix<u16>, lower_thresh_ratio: f64, upper_thresh_ratio: f64) -> (usize, usize, f64) {

		self.count(&matrix);
		let range = self.get_range(&matrix, lower_thresh_ratio, upper_thresh_ratio);
		let cog = self.get_center_of_gravity_ratio(range.0, range.1);
		
		(range.0, range.1, cog)
	}
	
	/**
	 * count the number of values per 'bin'
	 * re-uses the vec 'count'
	 */
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

	/**
	 * finds the range where values occur, 
	 * discounting the extreme values as described by lower/upper_thresh_ratio
	 */ 	
	fn get_range(&mut self, matrix: &Matrix<u16>, lower_thresh_ratio: f64, upper_thresh_ratio: f64) -> (usize, usize) {

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
	
	/**
	 * Returns a value in range (-1, +1) 
	 */
	fn get_center_of_gravity_ratio(&mut self, lower: usize, upper: usize) -> f64 {

		if lower == upper {
			return 0.0;
		}
		if upper == lower + 1 {
			return if self.counts[lower] < self.counts[upper] {
				return -1.0;
			} else {
				return 1.0;
			}
		}

		// get sum of all values
		let mut sum = 0u64;
		for i in lower..(upper + 1) {
			sum += self.counts[i] as u64;
		}
		
		// find index at the 16%, 50%, 84%
		let mut i_a = 0 as usize;
		let thresh = sum as f64 * (0.5 - 0.34);
		let mut s = 0;
		for i in lower..(upper + 1) {
			s += self.counts[i] as u64;
			if s as f64 > thresh {
				i_a = i;
				break;
			}
		}
		let mut i_b = 0 as usize;
		let thresh = sum as f64 * 0.5;
		let mut s = 0;
		for i in lower..(upper + 1) {
			s += self.counts[i] as u64;
			if s as f64 > thresh {
				i_b = i;
				break;
			}
		}
		let mut i_c = 0 as usize;
		let thresh = sum as f64 * (0.5 + 0.34);
		let mut s = 0;
		for i in lower..(upper + 1) {
			s += self.counts[i] as u64;
			if s as f64 > thresh {
				i_c = i;
				break;
			}
		}
		
		let a = math::map(i_a as f64, lower as f64, (upper - 1) as f64, -1.0, 1.0);
		let b = math::map(i_b as f64, lower as f64, (upper - 1) as f64, -1.0, 1.0);
		let c = math::map(i_c as f64, lower as f64, (upper - 1) as f64, -1.0, 1.0);
		return (a + b + c) / 3.0;  // good enuf
	}
}
