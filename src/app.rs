extern crate rustbox;
use std;
use vector2::Vector2f;
use math;
use animator::{Spec, Animator};
use input::Command;
use asciifier::Asciifier;
use textrenderer::TextRenderer;
use matrix::Matrix;
use mandelbrot;
use mandelbrot::Mandelbrot;
use pois::Pois;
use histogram::Histogram;

// is rough estimate of terminal font's character aspect ratio, which we can't rly know:
pub const CHARACTER_ASPECT_RATIO: f64 = 0.4;      

const DEG: f64 = std::f64::consts::PI / 180.0;

const ZOOM_INCREMENT: f64 = 0.015;
const VELOCITY_RATIO_INCREMENT: f64 = 0.007;
const ROTATION_VELOCITY_INCREMENT: f64 = 1.2 * DEG;
const TARGET_COEF: f64 = 0.08;
const FRICTION: f64 = 0.95;

const POSITION_EPSILON:f64 = 0.0005;
const SCALE_EPSILON: f64 = 0.00001;
const ROTATION_EPSILON:f64 = 0.00003;

const SHOW_DEBUG_TEXT: bool = false;


pub struct App<'a> {
	
    matrix: Matrix<u16>,
    histogram: Histogram,
    asciifier: Asciifier,
    renderer: TextRenderer<'a>,
    mandelbrot: Mandelbrot,
	pois: Pois,	
	
	view_width: usize,
	view_height: usize,
	max_escape: u16,
	count: u32,
	should_calculate: bool, 
	
	vp_center_anim: Animator<Vector2f>,
	vp_width_anim: Animator<f64>,
	vp_rotation_anim: Animator<f64>,
	asciifier_floor_anim: Animator<f64>,
	asciifier_ceil_anim: Animator<f64>,
	is_poi_anim: u8,  // 0 = no; 1 = zooming out; 2 = zooming in
	poi_index: usize,
	histogram_bias: f64,
	
	use_autoexposure: bool,
	has_shown_help: bool,
	help_anim: Animator<f64>,
	debug: String,
}


impl<'a> App<'a> {
	
	pub fn new() -> App<'a> {
		
	    let max_esc = mandelbrot::DEFAULT_MAX_ESCAPE;
	    let view_width = 80 as usize;
	    let view_height = 24 as usize;  
		
		App {
		    matrix: Matrix::new(view_width, view_height),
		    histogram: Histogram::new(max_esc as usize),
		    asciifier: Asciifier::new(0.0, max_esc as f64),
		    renderer: TextRenderer::new(view_width, view_height),
		    mandelbrot: Mandelbrot::new(max_esc, CHARACTER_ASPECT_RATIO, true),
			pois: Pois::new(),
		    
			vp_center_anim: Animator::<Vector2f>::new( Vector2f { x: 0.0, y: 0.0 }, Spec::None ),
			vp_width_anim: Animator::<f64>::new( mandelbrot::DEFAULT_WIDTH, Spec::None ),
			vp_rotation_anim: Animator::<f64>::new( 0.0, Spec::None ),
			asciifier_floor_anim: Animator::<f64>::new( 0.0, Spec::Target { target: 0.0, coefficient: 0.100, epsilon: None } ),
			asciifier_ceil_anim: Animator::<f64>::new( max_esc as f64, Spec::Target { target: max_esc as f64, coefficient: 0.100, epsilon: None } ),
			is_poi_anim: 0,
			poi_index: 0,
			histogram_bias: 0.0,
			
			view_width: view_width,
			view_height: view_height,
			max_escape: max_esc,
			count: 0,
			should_calculate: false,
			
			use_autoexposure: true,
			has_shown_help: false,
			help_anim: Animator::<f64>::new(1.0, Spec::None),
			debug: "".to_string()
		}
		
	    // note, size() should be called after instantiation with the real terminal dimensions
    }
	
	pub fn handle_command(&mut self, command: &Command) {

		let vel_increment = self.vp_width_anim.value as f64 * VELOCITY_RATIO_INCREMENT;  // abstract this

		match *command {
			Command::Poi(index) => {
				if ! (self.is_poi_anim > 0 && index == self.poi_index) {
					self.is_poi_anim = 1;  // 'phase 1' of 2: tween to home position
					self.poi_index = index;
					
					self.vp_center_anim.set_spec( Spec::Target {
							target: Vector2f { x: 0.0, y: 0.0 }, coefficient: TARGET_COEF * 0.4, epsilon: None } );  
					// ... will only get part-way to target position before 'phase 2' starts					
					
					self.vp_width_anim.set_spec( Spec::Target { 
							target: mandelbrot::DEFAULT_WIDTH, coefficient: TARGET_COEF * 1.5, epsilon: Some(0.003) } );
					// TODO: epsilon should be proportional to the size of 1 'pixel'
					
					// TODO: make exposure anim coef larger since the tween is fast, and then restore it afterwards
				}
			},
			Command::RotationVelocity(_) => { },
			Command::Help => { },
			_ => {
				// any command aside from the above turns off any 'poi anim'
				if self.is_poi_anim > 0 {
					self.is_poi_anim = 0;
					self.vp_center_anim.set_spec( Spec::None );
					self.vp_width_anim.set_spec( Spec::None ); 
				}
			}
		}

		match *command {
			
			Command::PositionVelocity(xm, ym) => {  
				let increment = Vector2f { x: vel_increment * xm, y: vel_increment * ym };
				
				match *self.vp_center_anim.spec() {
					Spec::VelocityWithRotation {  velocity, rotation, friction } => {
						self.vp_center_anim.set_spec( Spec::VelocityWithRotation { velocity: velocity + increment, rotation: rotation, friction: friction } );
					},
					_ => {
						self.vp_center_anim.set_spec(Spec::VelocityWithRotation { 
								velocity: increment, rotation: 0.0, friction: FRICTION });
					}
				};
			},

			Command::PositionTween(char_col, char_row) => {

				let screen_center_x = self.view_width as f64 / 2.0;
				let screen_offset_ratio_x = (char_col as f64 - screen_center_x) / screen_center_x;
				
				// y requires extra logic:
				let ar = self.view_width as f64 / self.view_height as f64;
				let viewport_height = self.vp_width_anim.value * (1.0 / ar)  *  (1.0 / self.mandelbrot.element_aspect_ratio);
				let screen_center_y = self.view_height as f64 / 2.0;
				let screen_offset_ratio_y = (char_row as f64 - screen_center_y) / screen_center_y;

				let vp_center = Vector2f::new(self.vp_width_anim.value / 2.0, viewport_height / 2.0);
				let vp_center_offset = Vector2f::new(screen_offset_ratio_x * vp_center.x, screen_offset_ratio_y * vp_center.y);
				
				let vp_center_offset = Vector2f::rotate(vp_center_offset, self.vp_rotation_anim.value);
				let target_x = self.vp_center_anim.value.x + vp_center_offset.x;
				let target_y = self.vp_center_anim.value.y + vp_center_offset.y;
				self.vp_center_anim = Animator::<Vector2f>::new(self.vp_center_anim.value, 
						Spec::Target {target: Vector2f { x: target_x, y: target_y }, coefficient: TARGET_COEF, epsilon: None });					
			}

			Command::Zoom(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				let current = match self.vp_width_anim.spec() {
					&Spec::ScaleVelocity { scale_velocity, .. } => scale_velocity,
					_ => 0.0,
				};
				self.vp_width_anim.set_spec( Spec::ScaleVelocity { 
						scale_velocity: current + increment, friction: FRICTION, epsilon: Some(SCALE_EPSILON) } );
			},
			
			Command::ZoomContinuous(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				self.vp_width_anim.set_spec( Spec::ScaleVelocity { 
						scale_velocity: increment, friction: 1.0, epsilon: Some(SCALE_EPSILON) } );
			},
			
			Command::RotationVelocity(multiplier) => {
				let increment = ROTATION_VELOCITY_INCREMENT * multiplier;
				match self.vp_rotation_anim.spec() {
					&Spec::Velocity { velocity, .. } => {
						self.vp_rotation_anim.set_spec( Spec::Velocity { 
								velocity: velocity + increment, friction: FRICTION, epsilon: Some(ROTATION_EPSILON) } );
					},
					_ => {
						self.vp_rotation_anim.set_spec( Spec::Velocity { 
								velocity: increment, friction: FRICTION, epsilon: Some(ROTATION_EPSILON) } );
					},
				}
			}

			Command::AutoExposure => self.use_autoexposure = ! self.use_autoexposure,
			
			Command::Stop => { 
				self.vp_center_anim.set_spec( Spec::None );
				self.vp_width_anim.set_spec( Spec::None ); 
				self.vp_rotation_anim.set_spec( Spec::None );
				self.is_poi_anim = 0;
			},
			Command::Reset => { 
				self.vp_center_anim.set_spec( 
						Spec::Target { target: Vector2f { x: 0.0, y: 0.0 }, coefficient: TARGET_COEF, epsilon: None });					
				self.vp_width_anim.set_spec( Spec::Target { 
						target: mandelbrot::DEFAULT_WIDTH, coefficient: TARGET_COEF, epsilon: Some(SCALE_EPSILON) } );
				self.vp_rotation_anim.value = math::normalize_theta(self.vp_rotation_anim.value);   
				self.vp_rotation_anim.set_spec( 
						Spec::Target { target: 0.0, coefficient: TARGET_COEF, epsilon: Some(ROTATION_EPSILON) } );
				self.is_poi_anim = 0;
			},

			Command::Size(w, h) => {
				self.size(w, h);
			}

			_ => {}
		}
		
		match *command {
			Command::Help => {
				
				if self.help_anim.value > 0.0 {
					self.has_shown_help = true;
					self.anim_in_help_dialog();
				} else {
					self.anim_out_help_dialog();
				}
			},
			_ => {
				if self.help_anim.value < 1.0 {
					self.anim_out_help_dialog();
				}
			}
		}
	}

	pub fn update(&mut self) {
		
		// width (zoom)
		let was_w = self.vp_width_anim.value;
		self.vp_width_anim.update();
		
		// rotation
		let was_rot = self.vp_rotation_anim.value;
		self.vp_rotation_anim.update();

		// and update center pos anim's rotation value 
		match self.vp_center_anim.spec() {
			&Spec::VelocityWithRotation { velocity, friction, .. } => {
				self.vp_center_anim.set_spec( Spec::VelocityWithRotation { 
						velocity: velocity, rotation: self.vp_rotation_anim.value, friction: friction  } );
			},
			_ => { }
		}

		// center pos
		let was_center = self.vp_center_anim.value.clone();
		self.vp_center_anim.update();
		
		// zero out (can't be done thru 'epsilon pattern' b/c dynamic)
		if self.is_poi_anim != 2 {
			let thresh = (self.vp_width_anim.value / self.view_width as f64) * POSITION_EPSILON;
			match self.vp_center_anim.spec() {
				&Spec::VelocityWithRotation { velocity, .. } => {
					if Vector2f::len(velocity) < thresh {
						self.vp_center_anim.set_spec(Spec::None);
					} 
				},
				&Spec::Target { target, .. } => {
					if Vector2f::len(target - self.vp_center_anim.value) < thresh {
						self.vp_center_anim.set_spec(Spec::None);
					} 
				}
				_ => { }
			}
		}

		
		// POI action
		if self.is_poi_anim == 1 { 
			match self.vp_width_anim.spec() {
				&Spec::None => {
					// the first anim has completed and the spec has reset itself to 'none'; do 'phase 2' of the anim
					self.is_poi_anim == 2;
					let poi = self.pois.get(self.poi_index);
					self.vp_center_anim.set_spec( Spec::Target {
							target: Vector2f { x: poi.0, y: poi.1 }, 
							coefficient: TARGET_COEF * 0.55, epsilon: None } );					
					let target_w = 1.0 / (poi.2 / mandelbrot::DEFAULT_WIDTH);
					self.vp_width_anim.set_spec( Spec::Target { 
							target: target_w, coefficient: TARGET_COEF * 0.5, epsilon: None } );  // don't clamp!
				},
				_ => { }
			}
		}
		
		self.asciifier_floor_anim.update();
		self.asciifier_ceil_anim.update();
		
		self.help_anim.update();
		

		let b =	was_w != self.vp_width_anim.value ||
				was_rot != self.vp_rotation_anim.value || 
				was_center != self.vp_center_anim.value;
		if b {
			self.should_calculate = true;
		}
		
		self.debug = if self.should_calculate { "".to_string() } else { "NOT DIRTY".to_string() }; 
	}
	
	pub fn calculate(&mut self) {

		if ! self.should_calculate {
			return;
		}
		
        self.mandelbrot.write_matrix(self.
        		vp_center_anim.value.clone(), self.vp_width_anim.value, self.vp_rotation_anim.value, 
        		&mut self.matrix);
        
		// calc histogram info
		let res = self.histogram.calc(&self.matrix, 0.040, 0.010);
		// TODO: this is awkward. how can i set target directly while using the spec getter? this applies everywhere else 
		self.asciifier_floor_anim.set_spec( Spec::Target 
				{ target: res.0 as f64, coefficient: 0.100, epsilon: None } ); 
		self.asciifier_ceil_anim.set_spec( Spec::Target 
				{ target: res.1 as f64, coefficient: 0.100, epsilon: None } );
		self.histogram_bias = if res.2 > 0.0 { 0.0 } else { res.2 };  // the positive values don't ever improve things rly
		
        self.should_calculate = false;
	}
	
	pub fn draw_frame(&mut self, debug_info: &String) {
	
		if self.use_autoexposure {
			self.asciifier.set_range(self.asciifier_floor_anim.value, self.asciifier_ceil_anim.value);
			self.asciifier.set_bias(self.histogram_bias);
		} else {
			self.asciifier.set_range(0.0, self.max_escape as f64);
			self.asciifier.set_bias(0.0);
		}

        self.renderer.draw_ascii_rect(&self.matrix, &self.asciifier);

        if SHOW_DEBUG_TEXT {
	        self.renderer.draw_string(&debug_info, 1, 1);
	        
			let s = format!(" rng {} to {} bias {:.*} ", 
					self.asciifier.floor() as usize, self.asciifier.ceil() as usize, 5, self.asciifier.bias());
	        self.renderer.draw_string(&s, 1,2);
	        self.renderer.draw_string(&self.debug, 1,3);
        }

        if self.count % 60 < 10 {  // show center-point
        	let x =  self.view_width / 2;
        	let y = self.view_height / 2;
	        self.renderer.draw_string(&"█".to_string(), x,y);	        	
        }
         
        if self.help_anim.value <= 1.0 {
        	let z = self.get_zoom();
        	self.renderer.draw_help_dialog(self.help_anim.value, &self.vp_center_anim.value, z);
        }

		if ! self.has_shown_help {
        	let s = " [H] help ".to_string();
        	self.renderer.draw_string(&s, self.view_width - s.len() - 1, 1);
		}
   
        self.renderer.render();
        
        self.count += 1;
	}
	
	fn size(&mut self, w: usize, h: usize) {
		self.view_width = w;
		self.view_height = h;
	    self.matrix = Matrix::new(self.view_width, self.view_height);
		self.renderer.size(self.view_width, self.view_height);
		self.should_calculate = true;
	}
	
	fn get_zoom(&self) -> f64 {
		mandelbrot::DEFAULT_WIDTH / self.vp_width_anim.value
		
	}

	//
	
	fn anim_in_help_dialog(&mut self) {
		self.help_anim = Animator::<f64>::new(self.help_anim.value, 
				Spec::Target { target: 0.0, coefficient: 0.20, epsilon: Some(0.01) });
	}
	
	fn anim_out_help_dialog(&mut self) {
		self.help_anim = Animator::<f64>::new(self.help_anim.value,
				Spec::Target { target: 1.0, coefficient: 0.20, epsilon: Some(0.01) }); 
	}
}
