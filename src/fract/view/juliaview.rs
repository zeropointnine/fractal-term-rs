extern crate num; 
extern crate num_cpus;

use self::num::complex::{Complex64};
use leelib::vector2::Vector2f;
use leelib::matrix::Matrix;
use leelib::animator::{Animator, Anim};
use leelib::dirtychecker::DirtyChecker;
use fract::constants;
use fract::fractalcalc::{FractalSpecs, FractalType};
use fract::Asciifier;
use fract::exposure::{ExposureInfo};
use fract::CoordList;
use fract::view::View;


pub struct JuliaView  {

	// members backed by trait getter/setters:
	specs: FractalSpecs,
    asciifier: Asciifier,
    fractal_matrix: Matrix<u16>,
    index_matrix: Matrix<u8>,

	position_animator: Animator<Vector2f>,
	width_animator: Animator<f64>,
	rotation_animator: Animator<f64>, 

    exposure_info: ExposureInfo, 
	exposure_floor_animator: Animator<f64>,
	exposure_ceil_animator: Animator<f64>,
	dirty_exposure_checker: DirtyChecker,
	use_exposure: bool,

	coord_anim_index: usize,
	coord_anim_phase: u8,

	debug:String,
	
	// struct-specific members:
	julia_coordlist: CoordList<Complex64>,
	julia_coord_animator: Animator<Vector2f>,
	dirty_fractal_checker: DirtyChecker, 
}

impl JuliaView {

	pub fn new(matrix_w: usize, matrix_h: usize, specs: FractalSpecs) -> Self {	

		JuliaView {  
			
			specs: specs,
		    asciifier: Asciifier::new(0.0, specs.max_val as f64),
		    fractal_matrix: Matrix::new(matrix_w, matrix_h),
		    index_matrix: Matrix::new(matrix_w, matrix_h),

			position_animator: Animator::<Vector2f>::new(Vector2f::new(0.0, 0.0), Anim::None),
			width_animator: Animator::<f64>::new(specs.default_width, Anim::None),
			rotation_animator: Animator::<f64>::new(0.0, Anim::None),
			dirty_fractal_checker: DirtyChecker::new(6),  // note, 2 more vals than mandelbrot version

			exposure_floor_animator: Animator::<f64>::new( 0.0, Anim::Target { target: 0.0, coefficient: 0.1, epsilon: Some(0.01) } ),
			exposure_ceil_animator: Animator::<f64>::new( specs.max_val as f64, Anim::Target { target: specs.max_val as f64, coefficient: 0.1, epsilon: Some(0.01) } ),
			exposure_info: ExposureInfo { floor: 0, ceil: specs.max_val as usize, bias: 0.0 },
			dirty_exposure_checker: DirtyChecker::new(2),
			use_exposure: true,

			coord_anim_phase: 0,
			coord_anim_index: 0,

			debug: "".to_string(),

			julia_coordlist: CoordList::<Complex64>::new(constants::JULIA_COMPLEX_TEXT),
			julia_coord_animator: Animator::<Vector2f>::new( Vector2f { x: 0.0, y: 0.0 }, Anim::None ),
		}
	}

	/**
	 * Tweens julia seed coordlist using julia_coord_animator
	 */
	fn start_julia_coord_animator(&mut self, current: Complex64, index: usize) {
		
		self.coord_anim_phase = 1;
		self.coord_anim_index = index;

		let current2 = Vector2f { x: current.re, y: current.im };  // convert for Animator
		self.julia_coord_animator.value = current2;
		
		let target = self.julia_coordlist.get(index);
		let target2 = Vector2f { x: target.re, y: target.im };  // convert for Animator
		let anim = Anim::Target { target: target2, coefficient: constants::TARGET_COEF * 1.0, epsilon: None };   
		self.julia_coord_animator.set_anim(anim);  
	}
}

impl View for JuliaView {

	fn specs(&self) -> &FractalSpecs {
		&self.specs
	}
	fn specs_m(&mut self) -> &mut FractalSpecs {
		&mut self.specs
	}

	fn fractal_matrix(&self) -> &Matrix<u16> {
		&self.fractal_matrix
	}
	fn fractal_matrix_m(&mut self) -> &mut Matrix<u16> {
		&mut self.fractal_matrix
	}

    fn index_matrix(&self) -> &Matrix<u8> {
    	&self.index_matrix
    }
    fn index_matrix_m(&mut self) -> &mut Matrix<u8> {
       	&mut self.index_matrix
    }

	fn set_matrix_size(&mut self, matrix_w: usize, matrix_h: usize) {
	    self.fractal_matrix = Matrix::new(matrix_w, matrix_h);
	    self.index_matrix = Matrix::new(matrix_w, matrix_h);
	    self.dirty_fractal_checker().force_dirty();
	}
	
    fn asciifier(&self) -> &Asciifier {
    	&self.asciifier
    }
	fn asciifier_m(&mut self) -> &mut Asciifier {
		&mut self.asciifier
	}
	
	fn position_animator(&mut self) -> &mut Animator<Vector2f> {
		&mut self.position_animator
	}
	fn width_animator(&mut self) -> &mut Animator<f64> {
		&mut self.width_animator
	}
	fn rotation_animator(&mut self) -> &mut Animator<f64> {
		&mut self.rotation_animator
	}
	fn dirty_fractal_checker(&mut self) -> &mut DirtyChecker {
		&mut self.dirty_fractal_checker
	}

	fn use_exposure(&self) -> bool {
		self.use_exposure
	}
	fn toggle_use_exposure(&mut self) {
		self.use_exposure = ! self.use_exposure;
	}
	fn exposure_info(&self) -> &ExposureInfo {
		&self.exposure_info
	}
	fn set_exposure_info(&mut self, info: ExposureInfo) {
		self.exposure_info = info;
	}	
	fn exposure_floor_animator(&mut self) -> &mut Animator<f64> {
		&mut self.exposure_floor_animator
	}
	fn exposure_ceil_animator(&mut self) -> &mut Animator<f64> {
		&mut self.exposure_ceil_animator
	}
	fn dirty_exposure_checker(&mut self) -> &mut DirtyChecker {
		&mut self.dirty_exposure_checker
	}
	
	fn coord_anim_phase(&self) -> u8 {
		self.coord_anim_phase
	}
	fn set_coord_anim_phase(&mut self, i: u8) {
		self.coord_anim_phase = i;
	}
	fn coord_anim_index(&self) -> usize {
		self.coord_anim_index
	}
	
	fn debug(&self) -> &String {
		&self.debug
	}
	fn set_debug(&mut self, s: String) {
		self.debug = s;
	}

	// ---	

	fn start_coord_anim(&mut self, index: usize) -> bool {
		if self.coord_anim_phase > 0 && index == self.coord_anim_index {
			false
		} else {
			match self.specs.fractal_type {
				FractalType::Julia(c) => {
					if index >= self.julia_coordlist.len() {
						false
					} else {
						self.start_julia_coord_animator(c, index);
						true
					}
				},
				_ => false
			}
		}
	}
	
	fn update(&mut self) {
		
		self.do_update();  // 'super'
		
		match self.specs.fractal_type {
			FractalType::Julia(ref mut c) => {
				if self.coord_anim_phase == 1 {
					match self.julia_coord_animator.anim() {
						&Anim::Target { .. } => {
							// update julia anim, and copy over value 
							self.julia_coord_animator.update();
							c.re = self.julia_coord_animator.value.x;
							c.im = self.julia_coord_animator.value.y;
						},
						_ => { }
					}
				}
			},
			_ => { }
		}		
	}
	
	fn do_dirty_fractal_check(&mut self) -> bool{
		let v = vec![self.position_animator.value.x, self.position_animator.value.x, 
			self.width_animator.value, self.rotation_animator.value,
			self.julia_coord_animator.value.x, self.julia_coord_animator.value.y];
		self.dirty_fractal_checker.do_check(v)
	}
}
