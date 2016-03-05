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
use fract::{CoordList, Three64};
use fract::view::View;


pub struct MandelView  {

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
	mandel_coordlist: CoordList<Three64>,
	dirty_fractal_checker: DirtyChecker, 
}

impl MandelView {

	pub fn new(matrix_w: usize, matrix_h: usize, specs: FractalSpecs) -> Self {	

		MandelView {
			specs: specs,
		    asciifier: Asciifier::new(0.0, specs.max_val as f64),
		    fractal_matrix: Matrix::new(matrix_w, matrix_h),
		    index_matrix: Matrix::new(matrix_w, matrix_h),

			position_animator: Animator::<Vector2f>::new(Vector2f::new(0.0, 0.0), Anim::None),
			width_animator: Animator::<f64>::new(specs.default_width, Anim::None),
			rotation_animator: Animator::<f64>::new(0.0, Anim::None),

			exposure_floor_animator: Animator::<f64>::new( 0.0, Anim::Target { target: 0.0, coefficient: 0.1, epsilon: Some(0.01) } ),
			exposure_ceil_animator: Animator::<f64>::new( specs.max_val as f64, Anim::Target { target: specs.max_val as f64, coefficient: 0.1, epsilon: Some(0.01) } ),
			exposure_info: ExposureInfo { floor: 0, ceil: specs.max_val as usize, bias: 0.0 },
			dirty_exposure_checker: DirtyChecker::new(2),
			use_exposure: true,

			coord_anim_phase: 0,
			coord_anim_index: 0,

			debug: "".to_string(),
			
			dirty_fractal_checker: DirtyChecker::new(4),
			mandel_coordlist: CoordList::<Three64>::new(constants::MANDELBROT_POI_TEXT),
		}
	}

	/**
	 * Zooms out to home position, and then tweens to specific coordlist + width,
	 * using the animators for position and width  
	 */
	fn start_mandel_coord_anim(&mut self, index: usize) {
		
		self.coord_anim_phase = 1;
		self.coord_anim_index = index;

		self.position_animator().set_anim( Anim::Target {
				target: Vector2f { x: 0.0, y: 0.0 }, coefficient: constants::TARGET_COEF * 0.4, epsilon: None } );  
		// ... will only get part-way to target position before 'phase 2' starts					
		
		let dw = self.specs.default_width;
		self.width_animator.set_anim( Anim::Target { 
				target: dw, coefficient: constants::TARGET_COEF * 1.5, epsilon: Some(0.003) } );
	}
	
	fn start_mandel_coord_anim_2(&mut self) {
		self.coord_anim_phase = 2;
		let poi = self.mandel_coordlist.get(self.coord_anim_index);
		self.position_animator().set_anim( Anim::Target {
				target: Vector2f { x: poi.0, y: poi.1 }, 
				coefficient: constants::TARGET_COEF * 0.55, epsilon: None } );					
		let target_w = 1.0 / (poi.2 / self.specs.default_width);
		self.width_animator().set_anim( Anim::Target { 
				target: target_w, coefficient: constants::TARGET_COEF * 0.5, epsilon: None } );  // important: no epsilon
	}
}

impl View for MandelView {

	fn fractal_matrix(&self) -> &Matrix<u16> {
		&self.fractal_matrix
	}
	fn fractal_matrix_m(&mut self) -> &mut Matrix<u16> {
		&mut self.fractal_matrix
	}

	fn specs(&self) -> &FractalSpecs {
		&self.specs
	}
	fn specs_m(&mut self) -> &mut FractalSpecs {
		&mut self.specs
	}

    fn index_matrix(&self) -> &Matrix<u8> {
    	&self.index_matrix
    }
    fn index_matrix_m(&mut self) -> &mut Matrix<u8> {
       	&mut self.index_matrix
    }

    fn asciifier(&self) -> &Asciifier {
    	&self.asciifier
    }
	fn asciifier_m(&mut self) -> &mut Asciifier {
		&mut self.asciifier
	}
	fn set_matrix_size(&mut self, matrix_w: usize, matrix_h: usize) {
	    self.fractal_matrix = Matrix::new(matrix_w, matrix_h);
	    self.index_matrix = Matrix::new(matrix_w, matrix_h);
	    self.dirty_fractal_checker().force_dirty();
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
				FractalType::Mandelbrot => {
					if index >= self.mandel_coordlist.len() {
						false
					} else {
						self.start_mandel_coord_anim(index);
						true
					}
				},
				_ => { false }
			}
		}
	}
	
	fn do_dirty_fractal_check(&mut self) -> bool {
		let v = vec![self.position_animator.value.x, self.position_animator.value.x, 
			self.width_animator.value, self.rotation_animator.value];
		self.dirty_fractal_checker.do_check(v)
	}
	
	fn update(&mut self) {
		
		self.do_update();  // 'super'
		
		match self.specs.fractal_type {
			FractalType::Mandelbrot => {
				if self.coord_anim_phase == 1 {
					match self.width_animator.anim() {
						&Anim::None => {
							// anim has finished, so start phase 2
							self.start_mandel_coord_anim_2();
						},
						_ => { }
					}
				} else if self.coord_anim_phase == 2 {
					match self.width_animator.anim() {
						&Anim::Target { target, .. } => {
							// end condition
							let thresh = (self.width_animator.value / self.fractal_matrix.width() as f64) * 0.1;
							let distance = (target - self.width_animator.value).abs();
							if distance  < thresh {
								self.width_animator.value = target;
								self.coord_anim_phase = 0;
							}
						},
						_ => { }
					}
				}
			},
			_ => { }
		}
	}
}
