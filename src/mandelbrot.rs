extern crate num;
extern crate num_cpus;

use std;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use vector2::Vector2f;
use matrix::Matrix;
use self::num::complex::Complex;
use self::num::traits::Float;


pub const DEFAULT_WIDTH: f64 = 3.5;  // a reasonable width that can display the main body of the Mandelbrot set
pub const DEFAULT_MAX_ESCAPE: u16 = 100;  


#[derive(Clone)]
pub struct Mandelbrot {
	pub max_escape: u16,  			// max escape value for Mandelbrot calculation (larger numbers = more expense) 
	pub element_aspect_ratio: f64,	// aspect ratio of each element ('pixel')
	pub use_multi_threads: bool, 	// use multi-threaded algorithm
	pub num_threads: usize,			// number of threads to use when use_multi_threads is true
}

impl Mandelbrot {
	
	pub fn new(max_escape: u16, element_aspect_ratio: f64, use_multi_threads: bool) -> Mandelbrot {
		Mandelbrot { 
			max_escape: max_escape,
			element_aspect_ratio: element_aspect_ratio,
			use_multi_threads: use_multi_threads,
			num_threads: num_cpus::get() as usize,
		}
	}

	pub fn write_matrix(&self, mandelbrot_center: Vector2f, mandelbrot_width: f64, rotation: f64, matrix: &mut Matrix<u16>) {
		if self.use_multi_threads {
			self.write_matrix_mt(mandelbrot_center, mandelbrot_width, rotation, matrix);
		} else {
			let h = matrix.height();
			self.write_matrix_section(mandelbrot_center, mandelbrot_width, rotation, matrix, 0, h);
		}
	} 
	 
	/**
	 * Calculate the Mandelbrot data in chunks handed off to separate threads
	 * Then, write the results to the passed-in matrix
	 */
	fn write_matrix_mt(&self, mandelbrot_center: Vector2f, mandelbrot_width: f64, rotation: f64, matrix: &mut Matrix<u16>) {

		// horizontal strips which make up the final mandelbrot data; 
		// the threads' 'work product' goes in here
		let sections: Vec<Matrix<u16>> = vec![Matrix::new(1,1); self.num_threads]; 

	    // make the data shareable and mutable
	    let wrapped_data = Arc::new(Mutex::new(sections));

		let (sender, receiver) = mpsc::channel::<bool>();

		let step = (matrix.height() as f64 / self.num_threads as f64).floor() as usize;
		for i in 0..self.num_threads {
			
			let start = i as usize * step;
			let end = if i < self.num_threads -1 {
				(i + 1) as usize * step
			} else {
				matrix.height()
			};
			let section_ht = end - start;
			// println!("i {} start {} end {} ht {}", i, start, end, section_ht);

	        // each thread needs its own sender instance
	        let sender = sender.clone();
	        // each thread needs its own thread-safe data reference
	        let wrapped_data = wrapped_data.clone();
			
			// note how we clone self b/c of use of instance method 
			let me = self.clone();	
			let matrix_w = matrix.width();
			let matrix_h = matrix.height();

	        thread::spawn(move || {

				let mut section: Matrix<u16> = Matrix::new(matrix_w, section_ht);			
				me.write_matrix_section(mandelbrot_center, mandelbrot_width, rotation, &mut section, start, matrix_h);

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
	 * Fills pre-existing 2d vector with mandelbrot set values
	 * 
	 * mandelbrot_width
	 *      the width in 'mandelbrot space' which will be mapped to the width of the matrix
	 * 		Note how height (in mandelbrot set's space) is derived from a combination of the  
	 * 		A/R of the full matrix height and element_aspect_ratio 
	 * mandelbrot_center
	 *      the center in 'mandelbrot space'
	 * section
	 *  	the matrix to be written to (which is a section of the full matrix)
	 * full_matrix_offset
	 *      the row from the full matrix where the section starts at
	 * full_matrix_height
	 *      height of the full matrix
	 */
	fn write_matrix_section(&self, 
			mandelbrot_center: Vector2f, mandelbrot_width: f64, rotation: f64, 
			section: &mut Matrix<u16>,  full_matrix_offset: usize, full_matrix_height: usize) {
		
		let mandelbrot_height = self.get_mandelbrot_height(section.width(), full_matrix_height, mandelbrot_width);
	
		let element_w = mandelbrot_width / section.width() as f64; 
		let element_h = mandelbrot_height / full_matrix_height as f64;

		let slope_x = Vector2f::rotate( Vector2f::new(element_w, 0.0), rotation );
		let slope_y = Vector2f::rotate( Vector2f::new(0.0, element_h), rotation );
		
		let half_matrix_w = section.width() as f64 / 2.0;
		let half_matrix_h = full_matrix_height as f64 / 2.0;

		for index_y in 0..section.height() {

			let mut cursor = mandelbrot_center;
			
			// move to left edge:
			let val = slope_x * -half_matrix_w;
			cursor =  cursor + val;
			
			// move 'up' or 'down' along left edge: 
			let val = slope_y * ((full_matrix_offset + index_y) as f64 - half_matrix_h);
			cursor = cursor + val; 
		 	
		 	for index_x in 0..section.width() {
		 		
				let value = self.get_value(cursor.x, cursor.y);					
	            section.set(index_x, index_y, value); 
				
				// move 'right'
				cursor.x += slope_x.x;
				cursor.y += slope_x.y;
		 	}
		}
	}

	fn get_mandelbrot_height(&self, matrix_width: usize, full_matrix_height: usize, mandelbrot_width: f64) -> f64 {
		let matrix_aspect_ratio = matrix_width as f64 / full_matrix_height as f64;
		let ht = mandelbrot_width * (1.0 / matrix_aspect_ratio)  *  (1.0 / self.element_aspect_ratio);
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

	// TEMP TEST
//	fn get_value_xxx(&self, x: f64, y: f64) -> u16 {
//		let c = Complex { re: x, im: y };
//
//        let i = 0..self.max_escape
//            c = (c * c) + c;
//        }
//        // If the threshold^2 is larger than the magnitude, return true.
//        return c.magnitude() < threshold*threshold;
//		
//
//
//
//		let mut z = Complex { re: 0f64, im: 0f64 };
//		let mut val = 0;
//		while z.norm_sqr().sqrt() < 2.0f64 && val < self.max_escape {   
//			z = z  * z + c;
//			val += 1;
//		}
//		val		
//	}
}
