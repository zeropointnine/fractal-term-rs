use leelib::math;
use leelib::matrix::Matrix;


pub struct ExposureInfo {
	pub floor: usize,
	pub ceil: usize,
	pub bias: f64
}


/**
 * 'Static' class
 * Finds the 'meaningful' range of values in a matrix, along with a 'bias' value.
 * Experiment.
 */ 
pub struct ExposureUtil;

impl ExposureUtil {
	
	/**
	 * max_val - the max value of anything in the matrix; used to create 'histogram'
	 * lower/upper_thresh_ratio - the ratio of the amount of upper and lower values to discard when calculating the range
	 * 
	 * returns the range where values occur, and the 'center of gravity' ratio (-1 to +1) within that range  
	 */ 
	pub fn calc(matrix: &Matrix<u16>, max_val: u16, lower_thresh_ratio: f64, upper_thresh_ratio: f64) -> ExposureInfo {

		// count the values in `matrix`
		let mut histogram = vec!(0u16; (max_val + 1) as usize);
		for val in matrix {
			histogram[val as usize] += 1;
		}

		let range = ExposureUtil::get_range(&histogram, &matrix, lower_thresh_ratio, upper_thresh_ratio);
		let bias = ExposureUtil::calc_bias(&histogram, range.0, range.1);
		ExposureInfo { floor: range.0, ceil: range.1, bias: bias }
	}
	
	/**
	 * Finds the range where values occur, 
	 * discounting the extreme values as described by lower/upper_thresh_ratio
	 */ 	
	fn get_range(histogram: &Vec<u16>, matrix: &Matrix<u16>, lower_thresh_ratio: f64, upper_thresh_ratio: f64) -> (usize, usize) {

		let sum_thresh =  (matrix.width() as f64 * matrix.height() as f64) * lower_thresh_ratio;
		let mut lower_index = 0;
		let mut sum = 0;
		for i in 0..histogram.len() {
			sum += histogram[i];
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
		for i in (0..histogram.len()).rev() {
			sum += histogram[i];
			if sum as f64 > sum_thresh {
				upper_index =  if i == histogram.len() - 1 { 
					histogram.len() - 1 
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
	fn calc_bias(histogram: &Vec<u16>, lower: usize, upper: usize) -> f64 {

		if lower == upper {
			return 0.0;
		}
		if upper == lower + 1 {
			return if histogram[lower] < histogram[upper] {
				return -1.0;
			} else {
				return 1.0;
			}
		}

		// get sum of all values
		let mut sum = 0u64;
		for i in lower..(upper + 1) {
			sum += histogram[i] as u64;
		}
		
		// find index at the 16%, 50%, 84%
		let mut i_a = 0 as usize;
		let thresh = sum as f64 * (0.5 - 0.34);
		let mut s = 0;
		for i in lower..(upper + 1) {
			s += histogram[i] as u64;
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
			s += histogram[i] as u64;
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
			s += histogram[i] as u64;
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
