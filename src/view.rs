extern crate num; 
extern crate num_cpus;

use constants;
use app;
use vector2::Vector2f;
use matrix::Matrix;
use animator::{Animator, Anim};
use fractalcalc::{FractalCalc, FractalSpecs, FractalType};
use asciifier::Asciifier;
use exposure::{ExposureUtil, ExposureInfo};
use coords::{Coords, Three64};
use self::num::complex::{Complex64};


/**
 * TODO: make trait for mandelbrot versus julia functionality, etc; this comes up around Coords behavior, etc
 */
pub struct View {

    pub matrix: Matrix<u16>,
    pub asciifier: Asciifier,
	pub specs: FractalSpecs,

	position_animator: Animator<Vector2f>,
	width_animator: Animator<f64>,
	rotation_animator: Animator<f64>, 

    exposure_util: ExposureUtil, 
    exposure_info: ExposureInfo, 
	exposure_floor_animator: Animator<f64>,
	exposure_ceil_animator: Animator<f64>,
	use_exposure: bool,

	// TODO: these items overlap; confusing; needs 'subclassing' to separate them logically
	coord_anim_index: usize,
	mandel_coords: Coords<Three64>,
	coord_anim_phase: u8,
	julia_coords: Coords<Complex64>,
	julia_coord_animator: Animator<Vector2f>,
	
	max_val: u16,

	last_pos: Vector2f,
	last_width: f64,
	last_rotation: f64,
	last_julia_coord: Vector2f,
	force_calc: bool,

	pub debug:String
}

impl View {

	pub fn new(matrix_w: usize, matrix_h: usize, specs: FractalSpecs) -> Self {	

		View {
			specs: specs,
			max_val: specs.max_val,
		    matrix: Matrix::new(matrix_w, matrix_h),

			position_animator: Animator::<Vector2f>::new(Vector2f::new(0.0, 0.0), Anim::None),
			width_animator: Animator::<f64>::new(specs.default_width, Anim::None),
			rotation_animator: Animator::<f64>::new(0.0, Anim::None),

		    asciifier: Asciifier::new(0.0, specs.max_val as f64),
			exposure_floor_animator: Animator::<f64>::new( 0.0, Anim::Target { target: 0.0, coefficient: 0.1, epsilon: None } ),
			exposure_ceil_animator: Animator::<f64>::new( specs.max_val as f64, Anim::Target { target: specs.max_val as f64, coefficient: 0.1, epsilon: None } ),

		    exposure_util: ExposureUtil::new(specs.max_val as usize),
			exposure_info: ExposureInfo { floor: 0, ceil: specs.max_val as usize, bias: 0.0 },
			use_exposure: true,

			coord_anim_phase: 0,
			coord_anim_index: 0,
			mandel_coords: Coords::<Three64>::new(constants::MANDELBROT_POI_TEXT),
			julia_coords: Coords::<Complex64>::new(constants::JULIA_COMPLEX_TEXT),
			julia_coord_animator: Animator::<Vector2f>::new( Vector2f { x: 0.0, y: 0.0 }, Anim::None ),
			
			last_pos: Vector2f::new(0.0, 0.0),
			last_width: 0.0,
			last_rotation: 0.0,
			last_julia_coord: Vector2f::new(0.0, 0.0),
			force_calc: true,
			
			debug: "".to_string()
		}
	}
	pub fn calculate(&mut self) {

		// self.force_calc

		if self.last_pos == self.position_animator.value && 
				self.last_width == self.width_animator.value && 
				self.last_rotation == self.rotation_animator.value &&
				self.last_julia_coord == self.julia_coord_animator.value && ! self.force_calc {
			return;
		}

		FractalCalc::calc_matrix(&self.specs, self.position_animator.value.clone(), self.width_animator.value, self.rotation_animator.value, &mut self.matrix);
		self.exposure_info = self.exposure_util.calc(&self.matrix, 0.040, 0.010);

		self.debug = format!(" exp {} {} {}", self.exposure_info.floor, self.exposure_info.ceil, self.exposure_info.bias);
		
		self.last_pos = self.position_animator.value;
		self.last_width = self.width_animator.value;
		self.last_rotation = self.rotation_animator.value;
		self.last_julia_coord = self.julia_coord_animator.value;
		self.force_calc = false;
	}
	
	pub fn update(&mut self) {

		// width (zoom)
		let orig = self.width_animator.value;
		self.width_animator.update();
		
		// width bounds check
		if self.width_animator.value > self.specs.default_width {
			self.width_animator.value = self.specs.default_width;
			match self.width_animator.anim() {
				&Anim::ScaleVelocity {  scale_velocity, .. } => { 
					self.width_animator.set_scale_velocity(scale_velocity.abs() * -0.25);  // muted bounce
				}
				_ => {},
			}
		}

		// rotation
		let orig = self.rotation_animator.value;
		self.rotation_animator.update();

		// and update position anim's rotation value 
		match self.position_animator.anim() {
			&Anim::VelocityWithRotation { .. } => {
				self.position_animator.set_vwr_rotation(self.rotation_animator.value);
			},
			_ => { }
		}
		
		// position
		let orig = self.position_animator.value.clone();
		self.position_animator.update();

		// position bounds check
		let mut b = false;
		let w = self.specs.default_width / 2.0;
		if self.position_animator.value.x < -w {
			self.position_animator.value.x = -w;
			b = true;
		}
		if self.position_animator.value.x > w {
			self.position_animator.value.x = w;
			b = true;
		}
		let h = FractalCalc::get_height(&self.specs, self.matrix.width(), self.matrix.height(), self.specs.default_width);
		let h = h / 2.0;
		if self.position_animator.value.y < -h {
			self.position_animator.value.y = -h;
			b = true;
		}
		if self.position_animator.value.y > h {
			self.position_animator.value.y = h;
			b = true;
		}
		if b {
			match self.position_animator.anim() {
				&Anim::VelocityWithRotation { velocity, .. } => {
					self.position_animator.set_vwr_velocity(velocity * -0.25);  // muted bounce
				},
				&Anim::Target { .. } => {
					self.position_animator.set_anim(Anim::None);  // does abrupt stop; good enough
				}
				_ => {}
			}
		}

		// coord anim 
		match self.specs.frac_type {
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
							let thresh = (self.width_animator.value / self.matrix.width() as f64) * 0.1;
							let distance = (target - self.width_animator.value).abs();
							self.debug = format!(" mandel thresh mult {} ", distance / thresh);
							if distance  < thresh {
								self.width_animator.value = target;
								self.coord_anim_phase = 0;
							}
						},
						_ => { }
					}
				}
			},
			FractalType::Julia {ref mut c} => {
				if self.coord_anim_phase == 1 {
					match self.julia_coord_animator.anim() {
						&Anim::Target { target, .. } => {
							// update julia anim, and copy over value 
							self.julia_coord_animator.update();
							c.re = self.julia_coord_animator.value.x;
							c.im = self.julia_coord_animator.value.y;
						},
						_ => { }
					}
				}
			}
		}

		// exposure 
		if self.use_exposure {
			self.exposure_floor_animator.set_target(self.exposure_info.floor as f64);
			self.exposure_ceil_animator.set_target(self.exposure_info.ceil as f64);
		} else {
			self.exposure_floor_animator.set_target(0.0);
			self.exposure_ceil_animator.set_target(self.max_val as f64);
		}
		self.exposure_floor_animator.update();
		self.exposure_ceil_animator.update();

		// apply exposure info to asciifer		
		self.asciifier.set_range(self.exposure_floor_animator.value, self.exposure_ceil_animator.value);
		let bias = if self.use_exposure {
			if self.exposure_info.bias > 0.0 { self.exposure_info.bias } else { 0.0 }
		} else {
			0.0
		};
		self.asciifier.set_bias(bias);  
	}
	
	pub fn set_matrix_size(&mut self, matrix_w: usize, matrix_h: usize) {
	    self.matrix = Matrix::new(matrix_w, matrix_h);
	    self.force_calc = true;
	}
	
	pub fn set_max_val(&mut self, max_val: u16) {
		self.max_val = max_val;
		self.exposure_util.set_max_val(max_val as usize);
		self.specs.max_val = max_val;
	}

	pub fn start_coord_anim(&mut self, index: usize) {
		if self.coord_anim_phase > 0 && index == self.coord_anim_index {
			return;
		}
		match self.specs.frac_type {
			FractalType::Mandelbrot => {
				if index >= self.mandel_coords.len() {
					return;
				}
				self.start_mandel_coord_anim(index);
			},
			FractalType::Julia {c} => {
				if index >= self.julia_coords.len() {
					return;
				}
				self.start_julia_coord_animator(c, index);
			}
		};
	}
	
	/**
	 * Zooms out to home position, and then tweens to specific coords + width,
	 * using the animators for position and width  
	 */
	fn start_mandel_coord_anim(&mut self, index: usize) {
		
		self.coord_anim_phase = 1;
		self.coord_anim_index = index;

		self.position_animator().set_anim( Anim::Target {
				target: Vector2f { x: 0.0, y: 0.0 }, coefficient: app::TARGET_COEF * 0.4, epsilon: None } );  
		// ... will only get part-way to target position before 'phase 2' starts					
		
		let dw = self.specs.default_width;
		self.width_animator.set_anim( Anim::Target { 
				target: dw, coefficient: app::TARGET_COEF * 1.5, epsilon: Some(0.003) } );
		// TODO: make exposure anim coef larger since the tween is fast, and then restore it afterwards
	}
	
	fn start_mandel_coord_anim_2(&mut self) {
		self.coord_anim_phase = 2;
		let poi = self.mandel_coords.get(self.coord_anim_index);
		self.position_animator().set_anim( Anim::Target {
				target: Vector2f { x: poi.0, y: poi.1 }, 
				coefficient: app::TARGET_COEF * 0.55, epsilon: None } );					
		let target_w = 1.0 / (poi.2 / self.specs.default_width);
		self.width_animator().set_anim( Anim::Target { 
				target: target_w, coefficient: app::TARGET_COEF * 0.5, epsilon: None } );  // important: no epsilon
	}
	
	/**
	 * Tweens julia seed coords using julia_coord_animator
	 */
	fn start_julia_coord_animator(&mut self, current: Complex64, index: usize) {
		
		self.coord_anim_phase = 1;
		self.coord_anim_index = index;

		let current2 = Vector2f { x: current.re, y: current.im };  // convert for Animator
		self.julia_coord_animator.value = current2;
		
		let target = self.julia_coords.get(index);
		let target2 = Vector2f { x: target.re, y: target.im };  // convert for Animator
		let anim = Anim::Target { target: target2, coefficient: app::TARGET_COEF * 1.0, epsilon: None };   
		self.julia_coord_animator.set_anim(anim);  
	}
	
	pub fn stop_coord_anim(&mut self) {
		if self.coord_anim_phase > 0 {
			self.coord_anim_phase = 0;
			self.position_animator.set_anim(Anim::None);
			self.width_animator.set_anim(Anim::None); 
		}
	}

	pub fn toggle_use_exposure(&mut self) {
		self.use_exposure = ! self.use_exposure;
		if self.use_exposure {
			self.exposure_floor_animator.set_target(0.0);
			self.exposure_ceil_animator.set_target(self.max_val as f64);
		}
	}
	
	pub fn element_aspect_ratio(&self) -> f64 {
		self.specs.element_ar
	}
	
	pub fn position_animator(&mut self) -> &mut Animator<Vector2f> {
		&mut self.position_animator
	}
	pub fn width_animator(&mut self) -> &mut Animator<f64> {
		&mut self.width_animator
	}
	pub fn rotation_animator(&mut self) -> &mut Animator<f64> {
		&mut self.rotation_animator
	}
}

/**
 * Simple vector wrapper of boxed Views 
 */
pub struct Views {
	pub vec: Vec<Box<View>>,
	pub index: usize,
}

impl Views {
	
	pub fn new() -> Self {
		Views { vec: Vec::new(), index:0 }
	}
	
	pub fn get(&mut self) -> &mut View {
		&mut (*self.vec[self.index])
	}
	pub fn get_im(&self) -> &View {
		&(*self.vec[self.index])
	}
	
	pub fn get_num(&mut self, i: usize) -> &mut View {
		&mut (*self.vec[i])
	}
	pub fn get_num_im(&self, i: usize) -> &View {
		&(*self.vec[i])
	}
}
