use matrix::Matrix;
use math;


pub struct ExposureInfo {
	pub floor: usize,
	pub ceil: usize,
	pub bias: f64
}


/**
 * Experiment - finds the 'meaningful' range of values in a matrix, along with a 'bias' value 
 * Has no state, aside from keeping 'bins' vec for re-use
 */ 
pub struct ExposureUtil {
	histogram: Vec<u16>  	
}

impl ExposureUtil {
	
	pub fn new(max_val: usize) -> ExposureUtil {
		let mut exp = ExposureUtil { histogram: vec!(u16::default(); 1) };
		exp.set_max_val(max_val);
		exp
	}
	
	pub fn set_max_val(&mut self, max_val: usize) {
		self.histogram = vec!(u16::default(); max_val + 1);
	}
	
	/**
	 * max_val - the max value of anything in the matrix; used to create 'histogram'
	 * lower/upper_thresh_ratio - the ratio of the amount of upper and lower values to discard when calculating the range
	 * 
	 * returns the range where values occur, and the 'center of gravity' ratio (-1 to +1) within that range  
	 */ 
	pub fn calc(&mut self, matrix: &Matrix<u16>, lower_thresh_ratio: f64, upper_thresh_ratio: f64) -> ExposureInfo {
		self.count(&matrix);		
		let range = self.get_range(&matrix, lower_thresh_ratio, upper_thresh_ratio);
		let cog = self.calc_bias(range.0, range.1);
		ExposureInfo { floor: range.0, ceil: range.1, bias: cog }
	}
	
	/**
	 * count the number of values per 'bin'
	 */
	fn count(&mut self, matrix: &Matrix<u16>) {
		for i in 0..(self.histogram.len()) {
			self.histogram[i] = 0;
		}
		for val in matrix {
			self.histogram[val as usize] += 1;
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
		for i in 0..self.histogram.len() {
			sum += self.histogram[i];
			if sum as f64 > sum_thresh {
				lower_index =  if i <= 1 { 
					0 as usize 
				} else { 
					i - 1  // rewind by 1
				};  
				break;
			}
		} 

		let sum_thresh =  (matrix.width() as f64 * matrix.height() as f64) * upper_thresh_ratio;
		let mut upper_index = 0;		
		let mut sum = 0;
		for i in (0..self.histogram.len()).rev() {
			sum += self.histogram[i];
			if sum as f64 > sum_thresh {
				upper_index =  if i == self.histogram.len() - 1 { 
					self.histogram.len() - 1 
				} else if i <= 1 { 
					0 
				} else { 
					i - 1 
				};
				break;
			}
		} 
		
		(lower_index, upper_index)
	}
	
	/**
	 * Returns a value in range (-1, +1) 
	 */
	fn calc_bias(&mut self, lower: usize, upper: usize) -> f64 {

		if lower == upper {
			return 0.0;
		}
		if upper == lower + 1 {
			return if self.histogram[lower] < self.histogram[upper] {
				return -1.0;
			} else {
				return 1.0;
			}
		}

		// get sum of all values
		let mut sum = 0u64;
		for i in lower..(upper + 1) {
			sum += self.histogram[i] as u64;
		}
		
		// find index at the 16%, 50%, 84%
		let mut i_a = 0 as usize;
		let thresh = sum as f64 * (0.5 - 0.34);
		let mut s = 0;
		for i in lower..(upper + 1) {
			s += self.histogram[i] as u64;
			if s as f64 > thresh {
				// is like 16th percentile; 1 standard deviation
				i_a = i;  
				break;
			}
		}
		let mut i_b = 0 as usize;
		let thresh = sum as f64 * 0.5;
		let mut s = 0;
		for i in lower..(upper + 1) {
			s += self.histogram[i] as u64;
			if s as f64 > thresh {
				// think 'center of gravity'
				i_b = i;  
				break;
			}
		}
		let mut i_c = 0 as usize;
		let thresh = sum as f64 * (0.5 + 0.34);
		let mut s = 0;
		for i in lower..(upper + 1) {
			s += self.histogram[i] as u64;
			if s as f64 > thresh {
				// is like 84th percentile
				i_c = i;  
				break;
			}
		}
		
		// make hand-wavey value using the above to represent 'bias'
		let a = math::map(i_a as f64, lower as f64, (upper - 1) as f64, -1.0, 1.0);
		let b = math::map(i_b as f64, lower as f64, (upper - 1) as f64, -1.0, 1.0);
		let c = math::map(i_c as f64, lower as f64, (upper - 1) as f64, -1.0, 1.0);
		return (a + b + c) / 3.0;
	}
}
