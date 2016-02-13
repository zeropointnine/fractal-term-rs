extern crate num;
use std::ops::Range;
use std::thread;
use std::sync::{Arc, Mutex};
use self::num::complex::Complex;
use self::num::traits::Float;
use math;
use vector2::Vector2f;


// a reasonable width that can fully display the main body of the Mandelbrot set 
pub const DEFAULT_WIDTH: f64 = 3.5;
pub const DEFAULT_MAX_ESCAPE: u16 = 100;  
pub const NUM_THREADS: u8 = 3;

pub type DataRow = Vec<u16>;
pub type DataMatrix = Vec<DataRow>;


pub struct Mandelbrot {
	pub max_escape: u16, 
	pub element_aspect_ratio: f64,
}

impl Mandelbrot {
	
	pub fn new(element_aspect_ratio: f64) -> Mandelbrot {
		Mandelbrot { 
			max_escape: DEFAULT_MAX_ESCAPE,
			element_aspect_ratio: element_aspect_ratio 
		}
	}

	/**
	 * Calculate and write the mandelbrot data to the matrix's full vertical range
	 */		
	pub fn write_matrix(&self, mandelbrot_center: &Vector2f, mandelbrot_width: f64, matrix: &mut DataMatrix) {
				
		let len = matrix.len();
		self.write_matrix_section(mandelbrot_center, mandelbrot_width, matrix, 0..len, len);
	}
	
	/**
	 * Multithreaded version
	 */
	pub fn write_matrix_mt(&self, mandelbrot_center: &Vector2f, mandelbrot_width: f64, matrix: &mut DataMatrix) {

		let matrix_width = matrix.len();
		let matrix_height = matrix[0].len();

		// let mut matrix = make_matrix_section(&self, mandelbrot_center, mandelbrot_width, num_cols, [0..num_rows], 
	}
	
	

	/**
	 * Fills in pre-existing 2d vector with mandelbrot set values
	 * 
	 * mandelbrot_width - the width in 'mandelbrot space' which will be mapped to the width of the matrix
	 * 		Note how height (in mandelbrot set's space) is derived from a combination of the  
	 * 		A/R of the full matrix height and element_aspect_ratio 
	 * mandelbrot_center - the center in 'mandelbrot space'
	 *
	 * matrix_row_range - the range of the 'data matrix' to be written to
	 * full_matrix_height - number of rows of the 'full' matrix
	 *
	 * The reason for the last two params is to hopefully  
	 */
	fn write_matrix_section(&self, 
			mandelbrot_center: &Vector2f, 
			mandelbrot_width: f64, 
			matrix: &mut DataMatrix,  
			matrix_row_range: Range<usize>, 
			full_matrix_height: usize) {
	
		let matrix_width = matrix[0].len();
		let mandelbrot_height = self.get_mandelbrot_height(matrix_width, full_matrix_height, mandelbrot_width);
		
	
		let x1 = mandelbrot_center.x - mandelbrot_width / 2.0;
		let x2 = mandelbrot_center.x + mandelbrot_width / 2.0;
		let y1 = mandelbrot_center.y - mandelbrot_height / 2.0;
		let y2 = mandelbrot_center.y + mandelbrot_height / 2.0;
	
		let offset = matrix_row_range.start;
	    for y in matrix_row_range {
	        for x in 0..matrix_width as usize {
	            let fx = math::map(x as f64,  0f64, matrix_width as f64,  x1, x2);
	            let fy = math::map(y as f64,  0f64, full_matrix_height as f64,  y1, y2);
	            matrix[y - offset][x] = self.get_value(fx, fy);;
	        }
	    }
	}
			
	fn get_mandelbrot_height(&self, matrix_width: usize, full_matrix_height: usize, mandelbrot_width: f64) -> f64 {
		let ratio = matrix_width as f64 / full_matrix_height as f64;
		let ht = mandelbrot_width * (1.0 / ratio)  *  (1.0 / self.element_aspect_ratio);
		ht		
	}		
	

	/**
	 * Returns a new matrix instance 
	 */
	pub fn make_matrix_section(&self, 
			mandelbrot_center: &Vector2f, 
			mandelbrot_width: f64, 
			matrix_width:usize,
			matrix_row_range: Range<usize>, 
			full_matrix_height: usize) -> DataMatrix {

		// make 2d vector instance	
		let mut matrix = DataMatrix::new();		
		let ht = matrix_row_range.end - matrix_row_range.start + 1;
		for _ in 0..ht {
			let col = vec!(0u16; matrix_width);
			matrix.push(col);
		}
	
		let matrix_width = matrix[0].len();
		
		let ratio = matrix_width as f64 / full_matrix_height as f64;
		let mandelbrot_height = mandelbrot_width * (1.0 / ratio)  *  (1.0 / self.element_aspect_ratio);
	
		let x1 = mandelbrot_center.x - mandelbrot_width / 2.0;
		let x2 = mandelbrot_center.x + mandelbrot_width / 2.0;
		let y1 = mandelbrot_center.y - mandelbrot_height / 2.0;
		let y2 = mandelbrot_center.y + mandelbrot_height / 2.0;
	
		let offset = matrix_row_range.start;
	    for y in matrix_row_range {
	        for x in 0..matrix_width as usize {
	            let fx = math::map(x as f64,  0f64, matrix_width as f64,  x1, x2);
	            let fy = math::map(y as f64,  0f64, full_matrix_height as f64,  y1, y2);
	            matrix[y - offset][x] = self.get_value(fx, fy);;
	        }
	    }
	    
	    matrix
	}

	/**
	 * The actual mandelbrot set calculation
	 */	
	fn get_value(&self, x: f64, y: f64) -> u16 {
		let c = Complex { re: x, im: y };
		let mut z = Complex { re: 0f64, im: 0f64 };
		let mut val = 0;
		while z.norm_sqr().sqrt() < 2.0f64 && val < self.max_escape {   
			z = z  * z + c;
			val += 1;
		}
		val
	}


	// TODO: Not done! May need to start over on this. Revisit
	// 
	pub fn write_matrix_multithreaded_orig(&self, mandelbrot_center: &Vector2f, mandelbrot_width: f64, matrix: &mut DataMatrix) {
	
		let num_rows = matrix.len();
		let num_cols = matrix[0].len();

		// TODO: wasteful. figure out how to make chunks a member variable (compiler complains)
		// let b = self.chunks.len() == 0 || self.chunks[0][0].len() != matrix[0].len();

		let mut r1;
		let mut r2 = 0;

		let mut chunks = Vec::<DataMatrix>::new();
		let rows_per_chunk = (num_rows as f64 / NUM_THREADS as f64).ceil() as usize;

		for i in 0..NUM_THREADS {

			r1 = r2;
			r2 = r1 + rows_per_chunk;
			if r2 > num_rows {
				r2 = num_rows;
			}

			let mut chunk: DataMatrix = Vec::new();		

			for _ in r1..r2 {
				let row = vec!(0u16; num_cols);
				chunk.push(row);
			}
			chunks.push(chunk);
		}
		
		r1 = 0;
		r2 = 0;
		let center = mandelbrot_center.clone();
		
		for i in 0..NUM_THREADS {

	        thread::spawn(move || {
	        	
				r1 = r2;
				r2 = r1 + rows_per_chunk;
				if r2 > num_rows {
					r2 = num_rows;
				}
				
				// TODO: make chunk...
				// TODO: phone home that thread is done
				
	        });
		}
		
		// do blocking method that waits for 4 thread-complete 'messages'
		
		// copy the data from 'chunks' into 'matrix'
		let mut dest_row_index = 0 as usize;
		for i in 0..NUM_THREADS as usize {
			let src_matrix = &chunks[i];
			for src_row in src_matrix {
				for i in 0..src_row.len() {
					matrix[dest_row_index][i] = src_row[i];
				}
				dest_row_index += 1;
			}
		}
	}
}
