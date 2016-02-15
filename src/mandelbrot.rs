extern crate num;
extern crate num_cpus;

use std::ops::Range;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use math;
use vector2::Vector2f;
use matrix::Matrix;
use self::num::complex::Complex;
use self::num::traits::Float;


pub const DEFAULT_WIDTH: f64 = 3.5;  // a reasonable width that can fully display the main body of the Mandelbrot set
pub const DEFAULT_MAX_ESCAPE: u16 = 50;  


#[derive(Clone)]
pub struct Mandelbrot {
	pub max_escape: u16,  			// max escape value for Mandelbrot calculation (larger numbers = more expense) 
	pub element_aspect_ratio: f64,	// aspect ratio of each element ('pixel')
	pub use_multi_threads: bool, 	// use multi-threaded algorithm
	pub num_threads: usize			// number of threads to use when use_multi_threads is true
}

impl Mandelbrot {
	
	pub fn new(element_aspect_ratio: f64) -> Mandelbrot {
		Mandelbrot { 
			max_escape: DEFAULT_MAX_ESCAPE,
			element_aspect_ratio: element_aspect_ratio,
			use_multi_threads: true,
			num_threads: num_cpus::get() as usize
		}
	}

	pub fn write_matrix(&self, mandelbrot_center: Vector2f, mandelbrot_width: f64, matrix: &mut Matrix<u16>) {

		if self.use_multi_threads {
			self.write_matrix_mt(mandelbrot_center, mandelbrot_width, matrix);
		} else {
			// calculate and write the mandelbrot data to the passed-in matrix 
			// (synchronous on the main thread)
			let h = matrix.height();
			self.write_matrix_section(mandelbrot_center, mandelbrot_width, matrix, 0..h, h);
		}
	} 
	 
	/**
	 * Calculate the Mandelbrot data in chunks handed off to separate threads
	 * Then, write the results to the passed-in matrix
	 */
	fn write_matrix_mt(&self, mandelbrot_center: Vector2f, mandelbrot_width: f64, matrix: &mut Matrix<u16>) {

		// horizontal strips which make up the final mandelbrot data; 
		// the 'work product' of the threads
		let sections: Vec<Matrix<u16>> = vec![Matrix::new(1,1); self.num_threads]; 

	    // make the data shareable and mutable
	    let wrapped_data = Arc::new(Mutex::new(sections));

		let (sender, receiver) = mpsc::channel::<bool>();

		let max_section_ht = (matrix.height() as f64 / self.num_threads as f64).ceil() as usize;
		for i in 0..self.num_threads {
			
			let start = i as usize * max_section_ht;
			let mut end = (i + 1) as usize * max_section_ht;
			if end >= matrix.height() {
				end = matrix.height();
			}
			let section_len = end - start;

	        // each thread needs its own sender instance
	        let sender = sender.clone();
	        // each thread needs its own thread-safe data reference
	        let wrapped_data = wrapped_data.clone();
			
			let matrix_w = matrix.width();
			let matrix_h = matrix.height();

			// note how we clone self itself b/c of use of instance method 
			let me = self.clone();	

	        thread::spawn(move || {

				let mut section: Matrix<u16> = Matrix::new(matrix_w, section_len);			
				me.write_matrix_section(mandelbrot_center, mandelbrot_width, &mut section, start..end, matrix_h);

                let mut locked_data = wrapped_data.lock().unwrap();
				locked_data[i] = section;

                let _ = sender.send(true);
	        });
		}
		
		let mut count = 0;
		loop {
	        // this blocks until the channel receiver gets a message
	        let _ = receiver.recv();
	        count += 1;
	        if count == self.num_threads {
	        	break;
	        }
		}

		// copy the chunks into the passed-in matrix		
        let locked_data = wrapped_data.lock().unwrap();
		let mut yoff: usize = 0;		
		for i in 0..locked_data.len() {   
			let section = &locked_data[i];
			matrix.copy_from(&section, yoff);
			yoff += section.height();
		}
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
			mandelbrot_center: Vector2f, 
			mandelbrot_width: f64, 
			matrix: &mut Matrix<u16>,  
			matrix_row_range: Range<usize>, 
			full_matrix_height: usize) {
	
		let mandelbrot_height = self.get_mandelbrot_height(matrix.width(), full_matrix_height, mandelbrot_width);
	
		let x1 = mandelbrot_center.x - mandelbrot_width / 2.0;
		let x2 = mandelbrot_center.x + mandelbrot_width / 2.0;
		let y1 = mandelbrot_center.y - mandelbrot_height / 2.0;
		let y2 = mandelbrot_center.y + mandelbrot_height / 2.0;
	
		let offset = matrix_row_range.start;
		
	    for y in matrix_row_range {
	        for x in 0..matrix.width() as usize {
	            let fx = math::map(x as f64,  0f64, matrix.width() as f64,  x1, x2);
	            let fy = math::map(y as f64,  0f64, full_matrix_height as f64,  y1, y2);
	            matrix.set(x, y - offset, self.get_value(fx, fy)); 
	        }
	    }
	}

	fn get_mandelbrot_height(&self, matrix_width: usize, full_matrix_height: usize, mandelbrot_width: f64) -> f64 {
		let ratio = matrix_width as f64 / full_matrix_height as f64;
		let ht = mandelbrot_width * (1.0 / ratio)  *  (1.0 / self.element_aspect_ratio);
		ht		
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
}
