extern crate rustbox;
extern crate num; 
use std;
use constants;
use vector2::Vector2f;
use math;
use animator::{Anim, Animator};
use input::Command;
use textbuffer::TextBuffer;
use view::{View, Views};
use fractalcalc::{FractalSpecs, FractalType};
use coords::Coords;
use matrix::Matrix;
use self::num::complex::{Complex64};


// rough estimate of terminal character a/r, which we can't rly know
pub const CHARACTER_ASPECT_RATIO: f64 = 0.4;

const DEG: f64 = std::f64::consts::PI / 180.0;

const ZOOM_INCREMENT: f64 = 0.015;
const VELOCITY_RATIO_INCREMENT: f64 = 0.007;
const ROTATIONAL_VELOCITY_INCREMENT: f64 = 1.2 * DEG;
pub const TARGET_COEF: f64 = 0.08;
const FRICTION: f64 = 0.95;

const SHOW_DEBUG_TEXT: bool = false;


pub struct App<'a> {

	views: Views,
    text_buffer: TextBuffer<'a>,

	view_width: usize,
	view_height: usize,

	has_shown_help: bool,
	help_anim: Animator<f64>,

	interview_animator: Animator<f64>,
	interview_matrix: Matrix<u16>,
	last_index: usize,

	count: u32,
}


impl<'a> App<'a> {
	
	pub fn new() -> App<'a> {
		
	    let view_width = 80 as usize;
	    let view_height = 24 as usize;  

		let mut app = App {
			views: Views::new(),
		    text_buffer: TextBuffer::new(view_width, view_height),
			view_width: view_width,
			view_height: view_height,
			
			has_shown_help: false,
			help_anim: Animator::<f64>::new(1.0, Anim::None),
			
			interview_animator: Animator::<f64>::new(1.0, Anim::None),
			interview_matrix: Matrix::new(view_width, view_height),
			last_index: 0,
			
			count: 0,
		};
		
		let m1 = View::new(view_width, view_height, FractalSpecs::new_mandelbrot_with_defaults(CHARACTER_ASPECT_RATIO));
		app.views.vec.push(Box::new(m1));

		let default_julia_coord;
		{
			let julia_coords = Coords::<Complex64>::new(constants::JULIA_COMPLEX_TEXT);
			default_julia_coord = julia_coords.get(1).clone();
		}
		let m2 = View::new(view_width, view_height, FractalSpecs::new_julia(default_julia_coord, CHARACTER_ASPECT_RATIO));
		app.views.vec.push(Box::new(m2));
		
		app.views.index = 0;

		app
	    // ... note, set_size() must be called after instantiation, with the real terminal dimensions
    }

//	fn view(&mut self) -> &mut View {
//		&mut self.views.get()  // wow!
//	}
	
	pub fn handle_command(&mut self, command: &Command) {

		let vel_increment = self.views.get().width_animator().value as f64 * VELOCITY_RATIO_INCREMENT;  // abstract this

		// coord anim, start and stop
		match self.views.get().specs.frac_type {
			FractalType::Mandelbrot => {
				match *command {
					Command::Coord(index) => {
						self.views.get().start_coord_anim(index);
					},
					Command::RotationalVelocity(_) | Command::AutoExposure | Command::Help | Command::Size(..) => {} 
					_ => {
						// any command aside from the above turns off coord anim 
						self.views.get().stop_coord_anim();
					}
				}
			},
			FractalType::Julia {..} => {
				match *command {
					Command::Coord(index) => {
						self.views.get().start_coord_anim(index);
					},
					Command::Reset | Command::Stop | Command::ChangeFractalSet => {
						self.views.get().stop_coord_anim();
					},
					_ => { }
				}
			}
		} 

		// main command match logic		
		match *command {
			Command::PositionVelocity(xm, ym) => {  
				let increment = Vector2f { x: vel_increment * xm, y: vel_increment * ym };
				
				match *self.views.get().position_animator().anim() {
					Anim::VelocityWithRotation {  velocity, .. } => {
						self.views.get().position_animator().set_vwr_velocity(velocity + increment);
					},
					_ => {
						self.views.get().position_animator().set_anim(Anim::VelocityWithRotation { 
								velocity: increment, rotation: 0.0, friction: FRICTION });
					}
				};
			},
			Command::PositionTween(char_col, char_row) => {

				let screen_center_x = self.view_width as f64 / 2.0;
				let screen_offset_ratio_x = (char_col as f64 - screen_center_x) / screen_center_x;
				
				// y requires extra logic:
				let ar = self.view_width as f64 / self.view_height as f64;
				let viewport_height = self.views.get().width_animator().value * (1.0 / ar)  *  (1.0 / self.views.get().element_aspect_ratio());
				let screen_center_y = self.view_height as f64 / 2.0;
				let screen_offset_ratio_y = (char_row as f64 - screen_center_y) / screen_center_y;

				let vp_center = Vector2f::new(self.views.get().width_animator().value / 2.0, viewport_height / 2.0);
				let vp_center_offset = Vector2f::new(screen_offset_ratio_x * vp_center.x, screen_offset_ratio_y * vp_center.y);
				
				let vp_center_offset = Vector2f::rotate(vp_center_offset, self.views.get().rotation_animator().value);
				let target_x = self.views.get().position_animator().value.x + vp_center_offset.x;
				let target_y = self.views.get().position_animator().value.y + vp_center_offset.y;
				self.views.get().position_animator().set_anim(
						Anim::Target {target: Vector2f { x: target_x, y: target_y }, coefficient: TARGET_COEF, epsilon: None } );
			}
			Command::Zoom(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				let current = match self.views.get().width_animator().anim() {
					&Anim::ScaleVelocity { scale_velocity, .. } => scale_velocity,
					_ => 0.0,
				};
				self.views.get().width_animator().set_anim( Anim::ScaleVelocity { 
						scale_velocity: current + increment, friction: FRICTION, epsilon: None } );
			},
			Command::ZoomContinuous(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				self.views.get().width_animator().set_anim( Anim::ScaleVelocity { 
						scale_velocity: increment, friction: 1.0, epsilon: None } );
			},
			Command::RotationalVelocity(multiplier) => {
				let increment = ROTATIONAL_VELOCITY_INCREMENT * multiplier;
				match self.views.get().rotation_animator().anim() {
					&Anim::Velocity { velocity, .. } => {
						self.views.get().rotation_animator().set_velocity(velocity + increment);
					},
					_ => {
						self.views.get().rotation_animator().set_anim( Anim::Velocity { 
								velocity: increment, friction: FRICTION, epsilon: None } );
					},
				}
			}
			Command::Stop => { 
				self.stop_view_anims();
			},
			Command::Reset => { 
				self.views.get().position_animator().set_anim( 
						Anim::Target { target: Vector2f { x: 0.0, y: 0.0 }, coefficient: TARGET_COEF, epsilon: None });					
				let dw = self.views.get().specs.default_width;
				self.views.get().width_animator().set_anim( Anim::Target { 
						target: dw, coefficient: TARGET_COEF, epsilon: None } );
				self.views.get().rotation_animator().value = math::normalize_theta(self.views.get().rotation_animator().value);   
				self.views.get().rotation_animator().set_anim( 
						Anim::Target { target: 0.0, coefficient: TARGET_COEF, epsilon: None } );
				self.views.get().stop_coord_anim();
			},
			Command::AutoExposure => { 
				self.views.get().toggle_use_exposure();
			} 
			Command::Size(w, h) => {
				self.set_size(w, h);
			},
			Command::Help => {
				if self.help_anim.value > 0.0 {
					self.has_shown_help = true;
					self.anim_in_help_dialog();
				} else {
					self.anim_out_help_dialog();
				}
			},
			Command::ChangeFractalSet => {
				self.stop_view_anims();
				self.last_index = self.views.index;
				self.views.index += 1;
				if self.views.index >= self.views.vec.len() {
					self.views.index = 0;
				}
				self.interview_animator.value = 0.0;
				self.interview_animator.set_anim(
						Anim::Velocity { velocity: 1.0/20.0, friction: 1.0, epsilon: None }); 
			}
			_ => {}
		}
	}

	fn stop_view_anims(&mut self) {
		self.views.get().position_animator().set_anim( Anim::None );
		self.views.get().width_animator().set_anim( Anim::None ); 
		self.views.get().rotation_animator().set_anim( Anim::None );
		self.views.get().stop_coord_anim();;
	}

	pub fn update(&mut self) {
		
		self.views.get().update();
		
		self.interview_animator.update();
		if self.interview_animator.value >= 1.0 {
			self.interview_animator.value = 0.0;
			self.interview_animator.set_anim(Anim::None);
		}
		
		self.help_anim.update();
		
	}
	
	pub fn calculate(&mut self) {
        self.views.get().calculate();
	}
	
	pub fn draw(&mut self, debug_info: &String) {
		
		let should_crossfade = match self.interview_animator.anim() { &Anim::None => false, _ => true };
		if should_crossfade {
			Matrix::interpolate(self.interview_animator.value,
					&self.views.get_num_im(self.last_index).matrix, 
					self.views.get_num_im(self.last_index).specs.max_val,
					&self.views.get_im().matrix, 
					self.views.get_im().specs.max_val,  
					&mut self.interview_matrix);
			self.text_buffer.draw_asciified_rect(&self.interview_matrix, &self.views.get_im().asciifier);
		} else {
	        self.text_buffer.draw_asciified_rect(&self.views.get_im().matrix, &self.views.get_im().asciifier);
		}

        if SHOW_DEBUG_TEXT {
	        self.text_buffer.draw_string(&debug_info, 1, 1);
	        self.text_buffer.draw_string(&self.views.get().debug, 1,2);
        }

        if self.count % 60 < 10 {  // show center-point
        	let x =  (self.view_width / 2) as i32;
        	let y = (self.view_height / 2) as i32;
	        self.text_buffer.draw_string(&"█".to_string(), x,y);	        	
        }
        
        if self.help_anim.value <= 1.0 {
        	let z = self.get_zoom();
        	let c = match self.views.get().specs.frac_type {  
        		FractalType::Mandelbrot => None,
        		FractalType::Julia {c} => Some(c),
        	};
        	self.text_buffer.draw_help_dialog(self.help_anim.value, &self.views.get().position_animator().value, z,  c);
        }

		if ! self.has_shown_help {
        	let s = " [H] help ".to_string();
        	self.text_buffer.draw_string(&s, (self.view_width - s.len() - 1) as i32, 1);
		}
   
        self.text_buffer.print();
        
        self.count += 1;
	}
	
	fn set_size(&mut self, w: usize, h: usize) {
		self.view_width = w;
		self.view_height = h;
		for i in 0..self.views.vec.len() {
			(*self.views.vec[i]).set_matrix_size(w, h);
		}
		self.text_buffer.set_size(self.view_width, self.view_height);
		self.interview_matrix = Matrix::new(self.view_width, self.view_height);
	}
	
	fn get_zoom(&mut self) -> f64 {
		let w2 = self.views.get().width_animator().value;
		let w1 = self.views.get().specs.default_width;
		w1 / w2
	}

	//
	
	fn anim_in_help_dialog(&mut self) {
		self.help_anim = Animator::<f64>::new(self.help_anim.value, 
				Anim::Target { target: 0.0, coefficient: 0.20, epsilon: Some(0.01) });
	}
	
	fn anim_out_help_dialog(&mut self) {
		self.help_anim = Animator::<f64>::new(self.help_anim.value,
				Anim::Target { target: 1.0, coefficient: 0.20, epsilon: Some(0.01) }); 
	}
}
