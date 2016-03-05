extern crate num;
extern crate num_cpus;

use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use self::num::complex::{Complex, Complex64};
use self::num::traits::Float;
use leelib::vector2::Vector2f;
use leelib::matrix::Matrix;


const DEFAULT_MANDELBROT_WIDTH: f64 = 4.0;
const DEFAULT_JULIA_WIDTH: f64 = 4.0; 


/**
 *
 */
#[derive(Clone, Copy)]
pub enum FractalType {
	Mandelbrot, 
	Julia(Complex64)
}


/**
 * Simple value object, passed around for use with FractalCalc methods
 */
#[derive(Clone, Copy)]
pub struct FractalSpecs {
	pub fractal_type: FractalType,
	pub max_val: u16,
	pub default_width: f64,
	pub default_center: Vector2f,
	pub element_ar: f64,
	pub num_threads: usize,
	pub use_multi_threads: bool,
}

impl FractalSpecs {
	pub fn new_mandelbrot_with_defaults(element_ar: f64) -> Self {
		FractalSpecs {
			fractal_type: FractalType::Mandelbrot,
			max_val: 500,
			default_width: DEFAULT_MANDELBROT_WIDTH,
			default_center: Vector2f::new(0.0, 0.0), 
			element_ar: element_ar,
			num_threads: num_cpus::get() as usize,
			use_multi_threads: true 
		}
	}
	
	pub fn new_julia(c: Complex64, element_ar: f64) -> Self {
		FractalSpecs {
			fractal_type: FractalType::Julia(c),

			max_val: 500,
			default_width: DEFAULT_JULIA_WIDTH,
			default_center: Vector2f::new(0.0, 0.0), 
			element_ar: element_ar,
			num_threads: num_cpus::get() as usize,
			use_multi_threads: true 
		}
	}
}


/**
 * 'Static' class
 * Fills in a `Matrix` with calculated fractal values
 */ 
pub struct FractalCalc;

impl FractalCalc {

	pub fn get_height(specs: &FractalSpecs, matrix_width: usize, full_matrix_height: usize, width: f64) -> f64 {
		let matrix_aspect_ratio = matrix_width as f64 / full_matrix_height as f64;
		let ht = width * (1.0 / matrix_aspect_ratio)  *  (1.0 / specs.element_ar);
		ht		
	}

	pub fn write_matrix(specs: &FractalSpecs, center: Vector2f, width: f64, rotation: f64, matrix: &mut Matrix<u16>) {
		if specs.use_multi_threads {
			FractalCalc::write_matrix_mt(&specs, center, width, rotation, matrix);
		} else {
			let h = matrix.height();
			FractalCalc::write_matrix_section(&specs, center, width, rotation, matrix, 0, h);
		}
	}

	/**
	 * Fills pre-existing 2d vector with mandelbrot set values
	 * 
	 * width
	 *      the width in 'mandelbrot space' which will be mapped to the width of the matrix
	 * 		Note how height (in mandelbrot set's space) is derived from a combination of the  
	 * 		A/R of the full matrix height and element_aspect_ratio 
	 * center
	 *      the center in 'mandelbrot space'
	 * section
	 *  	the matrix to be written to (which is a section of the full matrix)
	 * full_matrix_offset
	 *      the row from the full matrix where the section starts at
	 * full_matrix_height
	 *      height of the full matrix
	 */
	pub fn write_matrix_section(specs: &FractalSpecs, 
			center: Vector2f, width: f64, rotation: f64, 
			section: &mut Matrix<u16>,  full_matrix_offset: usize, full_matrix_height: usize) {
		
		let mandelbrot_height = FractalCalc::get_height(specs, section.width(), full_matrix_height, width);
	
		let element_w = width / section.width() as f64; 
		let element_h = mandelbrot_height / full_matrix_height as f64;

		let slope_x = Vector2f::rotate( Vector2f::new(element_w, 0.0), rotation );
		let slope_y = Vector2f::rotate( Vector2f::new(0.0, element_h), rotation );
		
		let half_matrix_w = section.width() as f64 / 2.0;
		let half_matrix_h = full_matrix_height as f64 / 2.0;

		for index_y in 0..section.height() {

			let mut cursor = center;
			
			// move to left edge:
			let val = slope_x * -half_matrix_w;
			cursor =  cursor + val;
			
			// move 'vertically' along 'left' edge: 
			let val = slope_y * ((full_matrix_offset + index_y) as f64 - half_matrix_h);
			cursor = cursor + val; 
		 	
		 	for index_x in 0..section.width() {

				let value = FractalCalc::get_value(&specs, cursor.x, cursor.y);
	            section.set(index_x, index_y, value); 
				
				// move 'right'
				cursor.x += slope_x.x;
				cursor.y += slope_x.y;
		 	}
		}
	}

	/**
	 * Calculate the MandelUtil data in chunks handed off to separate threads
	 * Then, write the results to the passed-in matrix
	 */
	fn write_matrix_mt(specs: &FractalSpecs, center: Vector2f, width: f64, rotation: f64, matrix: &mut Matrix<u16>) {

		// horizontal strips which make up the final fractal data; 
		// the threads' 'work product' goes in here
		let i = specs.num_threads;
		let sections: Vec<Matrix<u16>> = vec![Matrix::new(1,1); i]; 

	    // make the data shareable and mutable
	    let wrapped_data = Arc::new(Mutex::new(sections));

		let (sender, receiver) = mpsc::channel::<bool>();

		let step = (matrix.height() as f64 / specs.num_threads as f64).floor() as usize;
		for i in 0..specs.num_threads {
			
			let start = i as usize * step;
			let end = if i < specs.num_threads -1 {
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
			let spec = specs.clone();	
			let matrix_w = matrix.width();
			let matrix_h = matrix.height();

			
	        thread::spawn(move || {

				let mut section: Matrix<u16> = Matrix::new(matrix_w, section_ht);			
				FractalCalc::write_matrix_section(&spec, center, width, rotation, &mut section, start, matrix_h);

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
	        if count == specs.num_threads {
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

	pub fn get_value(specs: &FractalSpecs, x: f64, y: f64) -> u16 {
		// ersatz-dynamic dispatch (tried other refactoring routes which didn't work out :( )
		match specs.fractal_type {
			FractalType::Mandelbrot => FractalCalc::get_mandelbrot_value(x, y, specs.max_val),
			FractalType::Julia(c) => FractalCalc::get_julia_value(&c, x, y, specs.max_val),
		}
	}

	fn get_mandelbrot_value(x: f64, y: f64, max_val: u16) -> u16 {
		let c = Complex { re: x, im: y };
		let mut z = Complex { re: 0f64, im: 0f64 };
		let mut val = 0;
		while z.norm_sqr().sqrt() < 2.0f64 && val < max_val {   
			z = z  * z + c;
			val += 1;
		}
		val
	}
	
	fn get_julia_value(c: &Complex64, x: f64, y: f64, max_val: u16) -> u16 {
		let mut z = Complex{ re: x, im: y };
        for val in 0..max_val {
        	let z_abs:f64 = (z.re * z.re + z.im * z.im).sqrt();
            if z_abs > 2.0 {
            	return val;
            }
            z = z * z + c;
        }
        max_val
	}
}
