extern crate num; 
extern crate num_cpus;

use leelib::math;
use leelib::vector2::Vector2f;
use leelib::matrix::Matrix;
use leelib::animator::{Animator, Anim};
use leelib::dirtychecker::DirtyChecker;
use fract::constants;
use fract::fractalcalc::{FractalCalc, FractalSpecs};
use fract::Asciifier;
use fract::exposure::{ExposureUtil, ExposureInfo};


pub trait View {
	
	fn specs(&self) -> &FractalSpecs;
	fn specs_m(&mut self) -> &mut FractalSpecs;
    fn asciifier(&self) -> &Asciifier;
    fn asciifier_m(&mut self) -> &mut Asciifier;
	fn fractal_matrix(&self) -> &Matrix<u16>;
	fn fractal_matrix_m(&mut self) -> &mut Matrix<u16>;
    fn index_matrix(&self) -> &Matrix<u8>;
    fn index_matrix_m(&mut self) -> &mut Matrix<u8>;
	fn set_matrix_size(&mut self, matrix_w: usize, matrix_h: usize);
	
	fn position_animator(&mut self) -> &mut Animator<Vector2f>;
	fn width_animator(&mut self) -> &mut Animator<f64>;
	fn rotation_animator(&mut self) -> &mut Animator<f64>;
	fn dirty_fractal_checker(&mut self) -> &mut DirtyChecker;
	
	fn exposure_info(&self) -> &ExposureInfo;
	fn set_exposure_info(&mut self, info: ExposureInfo); 
	fn exposure_floor_animator(&mut self) -> &mut Animator<f64>;
	fn exposure_ceil_animator(&mut self) -> &mut Animator<f64>;
	fn use_exposure(&self) -> bool;
	fn toggle_use_exposure(&mut self);
	fn dirty_exposure_checker(&mut self) -> &mut DirtyChecker;

	fn set_exposure_anim_targets(&mut self, floor: f64, ceil: f64) {
		let a = Anim::Target { target: floor as f64, coefficient: 0.12, epsilon: Some(0.5) };
		self.exposure_floor_animator().set_anim(a);
		let a = Anim::Target { target: ceil as f64, coefficient: 0.12, epsilon: Some(0.5) };
		self.exposure_ceil_animator().set_anim(a);
	}
	
	fn coord_anim_index(&self) -> usize;
	fn coord_anim_phase(&self) -> u8;
	fn set_coord_anim_phase(&mut self, i: u8);
	
	fn start_coord_anim(&mut self, index: usize) -> bool;
	
	fn stop_coord_anim(&mut self) {
		if self.coord_anim_phase() > 0 {
			self.set_coord_anim_phase(0);
			self.position_animator().set_anim(Anim::None);
			self.width_animator().set_anim(Anim::None); 
		}
	}

	fn debug(&self) -> &String;
	fn set_debug(&mut self, s: String);
	
	//

	fn update(&mut self);
	
	fn do_update(&mut self) {

		// width (zoom)
		self.width_animator().update();
		
		// width bounds check
		let dw = self.specs().default_width;
		if self.width_animator().value > dw {
			self.width_animator().value = dw;
			match self.width_animator().anim() {
				&Anim::ScaleVelocity {  scale_velocity, .. } => { 
					self.width_animator().set_scale_velocity(scale_velocity.abs() * -0.25);  // muted bounce
				}
				_ => {},
			}
		}

		// rotation
		self.rotation_animator().update();

		// and update position anim's rotation value 
		match self.position_animator().anim() {
			&Anim::VelocityWithRotation { .. } => {
				let r = self.rotation_animator().value;
				self.position_animator().set_vwr_rotation(r);
			},
			_ => { }
		}
		
		// position
		self.position_animator().update();

		// position bounds check
		let mut b = false;
		let w = self.specs().default_width / 2.0;
		if self.position_animator().value.x < -w {
			self.position_animator().value.x = -w;
			b = true;
		}
		if self.position_animator().value.x > w {
			self.position_animator().value.x = w;
			b = true;
		}
		let mw = self.fractal_matrix().width();
		let mh = self.fractal_matrix().height();
		let dw = self.specs().default_width;
		let h = FractalCalc::get_height(&self.specs(), mw, mh, dw);
		let h = h / 2.0;
		if self.position_animator().value.y < -h {
			self.position_animator().value.y = -h;
			b = true;
		}
		if self.position_animator().value.y > h {
			self.position_animator().value.y = h;
			b = true;
		}
		if b {
			match self.position_animator().anim() {
				&Anim::VelocityWithRotation { velocity, .. } => {
					// muted bounce
					self.position_animator().set_vwr_velocity(velocity * -0.33);  
				},
				&Anim::Target { .. } => {
					// does abrupt stop; good enough
					self.position_animator().set_anim(Anim::None);  
				}
				_ => {}
			}
		}

		// exposure 
		let (f, c) = if self.use_exposure() {
			(self.exposure_info().floor as f64, self.exposure_info().ceil as f64)
		} else {
			(0.0, self.specs().max_val as f64)
		};
		self.set_exposure_anim_targets(f, c);
		self.exposure_floor_animator().update();
		self.exposure_ceil_animator().update();
	}
	
	fn do_dirty_fractal_check(&mut self) -> bool;

	fn calculate(&mut self) {

		let dirty1 = self.do_dirty_fractal_check();
		if dirty1 {
			// calc fractal matrix using positional info
			let pos = self.position_animator().value.clone();
			let w = self.width_animator().value;
			let r = self.rotation_animator().value;
			let specs = self.specs().clone();
			FractalCalc::write_matrix(&specs, pos, w, r, &mut self.fractal_matrix_m());
	
			// calc 'exposure info' from matrix
			let info = ExposureUtil::calc(&self.fractal_matrix(), self.specs().max_val, 0.040, 0.010);
			self.set_exposure_info(info);
		}

		// apply exposure info to asciifer		
		let f = self.exposure_floor_animator().value;
		let c = self.exposure_ceil_animator().value;
		self.asciifier_m().set_floor_ceil(f, c);

		let bias = if self.use_exposure() {
			if self.exposure_info().bias > 0.0 { self.exposure_info().bias } else { 0.0 }
		} else {
			0.0
		};
		self.asciifier_m().set_bias(bias);  

		let v = vec![self.exposure_floor_animator().value, self.exposure_ceil_animator().value];
		let dirty2 = self.dirty_exposure_checker().do_check(v);
		if  dirty1 || dirty2 {
			self.calc_index_matrix();
		}

		// self.set_debug(format!(" exp {} {} {}", self.exposure_info().floor, self.exposure_info().ceil, self.exposure_info().bias));
	}
	
	fn anim_to_home(&mut self) {
		self.stop_coord_anim();
		self.position_animator().set_anim( 
				Anim::Target { target: Vector2f { x: 0.0, y: 0.0 }, coefficient: constants::TARGET_COEF, epsilon: None });					
		let dw = self.specs().default_width;
		self.width_animator().set_anim( Anim::Target { 
				target: dw, coefficient: constants::TARGET_COEF, epsilon: None } );
		self.rotation_animator().value = math::normalize_theta(self.rotation_animator().value);   
		self.rotation_animator().set_anim( 
				Anim::Target { target: 0.0, coefficient: constants::TARGET_COEF, epsilon: None } );
	}
	
	/**
	 * Transforms fractal values in `fractal_matrix` to char index values in `index_matrix`, using `asciifier`
	 */
	// 'private'
	fn calc_index_matrix(&mut self) {  
		
		assert!(self.fractal_matrix().width() == self.index_matrix().width() && 
				self.fractal_matrix().height() == self.index_matrix().height());
		 
		for y in 0..self.fractal_matrix().height() {
			for x in 0..self.fractal_matrix().width() {
				let val =  self.fractal_matrix().get(x, y);
				let i = self.asciifier().to_char_index(val as f64);
				self.index_matrix_m().set(x, y, i);
			}
		}
	}
}	
