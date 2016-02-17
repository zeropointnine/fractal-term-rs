extern crate rustbox;
use std;
use vector2::Vector2f;
use animator::{Spec, Animator};
use input::Command;
use textrenderer::{TextRenderer, Asciifier};
use matrix::Matrix;
use mandelbrot;
use mandelbrot::Mandelbrot;

// rough estimate of terminal font's character aspect ratio, which we can't rly know
pub const CHARACTER_ASPECT_RATIO: f64 = 0.4;      
const DEG: f64 = std::f64::consts::PI / 180.0;
const ZOOM_INCREMENT: f64 = 0.015;
const VELOCITY_RATIO_INCREMENT: f64 = 0.007;
const ROTATION_VELOCITY_INCREMENT: f64 = 1.2 * DEG;
const TWEEN_MULTIPLIER: f64 = 0.08;
const FRICTION: f64 = 0.95;
static HELP_TEXT: &'static str = include_str!("help.txt");
const SHOW_DEBUG_TEXT: bool = true;


pub struct App<'a> {
	
    matrix: Matrix<u16>,
    renderer: TextRenderer,
    asciifier: Asciifier,
	
    mandelbrot: Mandelbrot,
	vp_center_anim: Animator<Vector2f>,
	vp_width_anim: Animator<f64>,
	vp_rotation_anim: Animator<f64>,

	view_width: usize,
	view_height: usize,
	max_escape: u16,
	
	count: u32,
	
	help_text: Vec<&'a str>,
	should_show_help: bool,
	has_shown_help: bool
}


impl<'a> App<'a> {
	
	pub fn new(view_width: usize, view_height: usize) -> App<'a> {
		
	    let max_esc = 500;
		
		let app = App {
		    matrix: Matrix::new(view_width, view_height),
		    renderer: TextRenderer::new(view_width, view_height),
		    asciifier: Asciifier::new(max_esc as f64),
		    
		    // spirals x: 0.32450637064523491, y:0.04855313313619743
		    // tip: x: -178644781511005024, y: 0.0
		    
		    mandelbrot: Mandelbrot::new(max_esc, CHARACTER_ASPECT_RATIO, true),
			vp_center_anim: Animator { value: Vector2f { x: 0.32450637064523491, y:0.04855313313619743 }, spec: Spec::None },
			vp_width_anim: Animator { value: mandelbrot::DEFAULT_WIDTH, spec: Spec::None },
			vp_rotation_anim: Animator { value: 0.0, spec: Spec::None },
			
			view_width: view_width,
			view_height: view_height,
			max_escape: max_esc,
			count: 0,

			help_text: HELP_TEXT.lines().collect(),
			should_show_help: false,
			has_shown_help: false,
		};
		
		app
    }
	
	pub fn handle_command(&mut self, command: &Command) {

		let vel_increment = self.vp_width_anim.value as f64 * VELOCITY_RATIO_INCREMENT;  // abstract this

		match *command {
			
			Command::PositionVelocity(xm, ym) => {  
				let increment = Vector2f { x: vel_increment * xm, y: vel_increment * ym };
				match self.vp_center_anim.spec {
					Spec::VelocityWithRotation { ref mut velocity, .. } => {
						*velocity = *velocity + increment;
					},
					_ => {
						self.vp_center_anim.spec = Spec::VelocityWithRotation { 
							velocity: increment, rotation: 0.0, friction: FRICTION
						} 
					}
				}
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
				
				self.vp_center_anim.spec = Spec::Tween {
					target: Vector2f { x: target_x, y: target_y }, coefficient: TWEEN_MULTIPLIER}					
			}

			Command::Zoom(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				let current_scale = match self.vp_width_anim.spec {
					Spec::Scale { scale, .. } => scale,
					_ => 0.0,
				};
				self.vp_width_anim.spec = Spec::Scale { scale: current_scale + increment, friction: FRICTION };
			},
			
			Command::ZoomContinuous(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				self.vp_width_anim.spec = Spec::Scale { scale: increment, friction: 1.0 };
			},
			
			Command::RotationVelocity(multiplier) => {
				let increment = ROTATION_VELOCITY_INCREMENT * multiplier;
				match self.vp_rotation_anim.spec {
					Spec::Velocity { ref mut velocity, .. } => {
						*velocity = *velocity + increment;
					},
					_ => {
						self.vp_rotation_anim.spec = Spec::Velocity { velocity: increment, friction: FRICTION }
					},
				}
			}

			Command::Resize(w, h) => {
				self.size(w, h);
			}

			Command::Stop => { 
				self.vp_center_anim.spec = Spec::None;
				self.vp_width_anim.spec = Spec::None; 
				self.vp_rotation_anim.spec = Spec::None;
			},
			Command::Reset => { 
				self.vp_center_anim.value.x = 0.0;
				self.vp_center_anim.value.y = 0.0;
				self.vp_center_anim.spec = Spec::None;
				self.vp_width_anim.value = mandelbrot::DEFAULT_WIDTH;
				self.vp_width_anim.spec = Spec::None;
				self.vp_rotation_anim.value = 0.0;
				self.vp_rotation_anim.spec = Spec::None;
			},

			_ => {}
		}
		
		match *command {
			Command::Help => {
				self.should_show_help = true;
				self.has_shown_help = true;
			},
			_ => {
				self.should_show_help = false;
			}
		}
	}
	
	pub fn get_magnification(&self) -> f64 {
		mandelbrot::DEFAULT_WIDTH / self.vp_width_anim.value
	}
	
	pub fn update(&mut self) {
		
		self.vp_width_anim.update();
		
		self.vp_rotation_anim.update();

		match self.vp_center_anim.spec {
			Spec::VelocityWithRotation { ref mut rotation, .. } => {
				*rotation = self.vp_rotation_anim.value;
			},
			_ => { }

		}
		self.vp_center_anim.update();
	}
	
	pub fn calculate(&mut self) {
        self.mandelbrot.write_matrix(self.
        		vp_center_anim.value.clone(), self.vp_width_anim.value, self.vp_rotation_anim.value, 
        		&mut self.matrix);
	}
	
	pub fn draw_frame(&mut self, debug_info: &String) {
        
        self.renderer.draw_ascii_rect(&self.matrix, &self.asciifier);

        let mut info = format!(" {:.0}x ", self.get_magnification());
        if SHOW_DEBUG_TEXT {
        	info = info + &debug_info;
        }    
        self.renderer.draw_string(&info, 1, self.view_height - 1);

        if self.count % 60 < 10 {  // show center-point
        	let x =  self.view_width / 2;
        	let y = self.view_height / 2;
	        self.renderer.draw_string(&"█".to_string(), x,y);	        	
        }
        
        if self.should_show_help {
        	self.renderer.draw_text(&self.help_text, &self.vp_center_anim.value);
        	
        }

		if ! self.should_show_help && ! self.has_shown_help {
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
}
