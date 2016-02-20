extern crate rustbox;
use std;
use vector2::Vector2f;
use animator::{Spec, Animator};
use input::Command;
use textrenderer::{TextRenderer, Asciifier};
use matrix::Matrix;
use mandelbrot;
use mandelbrot::Mandelbrot;
use pois;
use pois::Pois;

// rough estimate of terminal font's character aspect ratio, which we can't rly know
pub const CHARACTER_ASPECT_RATIO: f64 = 0.4;      
const DEG: f64 = std::f64::consts::PI / 180.0;
const ZOOM_INCREMENT: f64 = 0.015;
const VELOCITY_RATIO_INCREMENT: f64 = 0.007;
const ROTATION_VELOCITY_INCREMENT: f64 = 1.2 * DEG;
const BASE_TWEEN_COEF: f64 = 0.08;
const FRICTION: f64 = 0.95;
const SHOW_DEBUG_TEXT: bool = false;


pub struct App<'a> {
	
    matrix: Matrix<u16>,
    renderer: TextRenderer<'a>,
    asciifier: Asciifier,
	
    mandelbrot: Mandelbrot,
	vp_center_anim: Animator<Vector2f>,
	vp_width_anim: Animator<f64>,
	vp_rotation_anim: Animator<f64>,
	is_poi_anim: u8,  // 0 = no; 1 = zooming out; 2 = zooming in
	poi_index: usize,
	
	view_width: usize,
	view_height: usize,
	max_escape: u16,
	
	count: u32,

	pois: Pois,	
	has_shown_help: bool,
	help_anim: Animator<f64>,
}


impl<'a> App<'a> {
	
	pub fn new() -> App<'a> {
		
	    let max_esc = 1000;
	    let view_width = 80 as usize;
	    let view_height = 24 as usize;  
		
		App {
		    matrix: Matrix::new(view_width, view_height),
		    renderer: TextRenderer::new(view_width, view_height),
		    asciifier: Asciifier::new(max_esc as f64),
		    
		    mandelbrot: Mandelbrot::new(max_esc, CHARACTER_ASPECT_RATIO, true),
			vp_center_anim: Animator::<Vector2f>::new( Vector2f { x: 0.0, y: 0.0 }, Spec::None ),
			vp_width_anim: Animator::<f64>::new( mandelbrot::DEFAULT_WIDTH, Spec::None ),
			vp_rotation_anim: Animator::<f64>::new(0.0, Spec::None),
			is_poi_anim: 0,
			poi_index: 0,
			
			view_width: view_width,
			view_height: view_height,
			max_escape: max_esc,
			count: 0,

			pois: Pois::new(),
			has_shown_help: false,
			help_anim: Animator::<f64>::new(1.0, Spec::None)
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
							target: Vector2f { x: 0.0, y: 0.0 }, coefficient: BASE_TWEEN_COEF * 0.5, epsilon: None } );  
					// ... will only get part-way to target position before 'phase 2' starts					
					
					self.vp_width_anim.set_spec( Spec::Target { 
							target: mandelbrot::DEFAULT_WIDTH, coefficient: BASE_TWEEN_COEF * 2.5, epsilon: Some(0.003) } );
					// TODO: epsilon should be proportional to the size of 1 'pixel'
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
						Spec::Target {target: Vector2f { x: target_x, y: target_y }, coefficient: BASE_TWEEN_COEF, epsilon: None });					
			}

			Command::Zoom(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				let current_scale = match self.vp_width_anim.spec() {
					&Spec::Scale { scale, .. } => scale,
					_ => 0.0,
				};
				self.vp_width_anim.set_spec( Spec::Scale { scale: current_scale + increment, friction: FRICTION } );
			},
			
			Command::ZoomContinuous(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				self.vp_width_anim.set_spec( Spec::Scale { scale: increment, friction: 1.0 } );
			},
			
			Command::RotationVelocity(multiplier) => {
				let increment = ROTATION_VELOCITY_INCREMENT * multiplier;
				match self.vp_rotation_anim.spec() {
					&Spec::Velocity { velocity, .. } => {
						self.vp_rotation_anim.set_spec( Spec::Velocity { velocity: velocity + increment, friction: FRICTION } );
					},
					_ => {
						self.vp_rotation_anim.set_spec( Spec::Velocity { velocity: increment, friction: FRICTION } );
					},
				}
			}
			
			Command::Stop => { 
				self.vp_center_anim.set_spec( Spec::None );
				self.vp_width_anim.set_spec( Spec::None ); 
				self.vp_rotation_anim.set_spec( Spec::None );
				self.is_poi_anim = 0;
			},
			Command::Reset => { 
				self.vp_center_anim.value.x = 0.0;
				self.vp_center_anim.value.y = 0.0;
				self.vp_center_anim.set_spec( Spec::None );
				self.vp_width_anim.value = mandelbrot::DEFAULT_WIDTH;
				self.vp_width_anim.set_spec( Spec::None );
				self.vp_rotation_anim.value = 0.0;
				self.vp_rotation_anim.set_spec( Spec::None );
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
		
		self.vp_width_anim.update();

		self.vp_rotation_anim.update();
		match self.vp_center_anim.spec() {
			&Spec::VelocityWithRotation { velocity, rotation, friction } => {
				// update center pos anim's rotation value 
				self.vp_center_anim.set_spec( Spec::VelocityWithRotation { 
						velocity: velocity, rotation: self.vp_rotation_anim.value, friction: friction  } );
			},
			_ => { }
		}

		self.vp_center_anim.update();
		
		if self.is_poi_anim == 1 { 
			match self.vp_width_anim.spec() {
				&Spec::None => {
					// the first anim has completed and the spec has reset itself to 'none'; do 'phase 2' of the anim
					self.is_poi_anim == 2;
					let poi = self.pois.get(self.poi_index);
					self.vp_center_anim.set_spec( Spec::Target {
							target: Vector2f { x: poi.0, y: poi.1 }, 
							coefficient: BASE_TWEEN_COEF * 0.55, epsilon: None } );					
					let target_w = 1.0 / (poi.2 / mandelbrot::DEFAULT_WIDTH);
					self.vp_width_anim.set_spec( Spec::Target { 
							target: target_w, coefficient: BASE_TWEEN_COEF * 0.5, epsilon: None } );
				},
				_ => { }
			}
		}
		
		self.help_anim.update();
	}
	
	pub fn calculate(&mut self) {
        self.mandelbrot.write_matrix(self.
        		vp_center_anim.value.clone(), self.vp_width_anim.value, self.vp_rotation_anim.value, 
        		&mut self.matrix);
	}
	
	pub fn draw_frame(&mut self, debug_info: &String) {
        
        self.renderer.draw_ascii_rect(&self.matrix, &self.asciifier);

        if SHOW_DEBUG_TEXT {
	        self.renderer.draw_string(&debug_info, 1, self.view_height - 1);
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
