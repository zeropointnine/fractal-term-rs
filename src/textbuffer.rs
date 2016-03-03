extern crate num; 
use std::cmp::{max, min};
use ansi;
use matrix::Matrix;
use vector2::Vector2f;
use asciifier::Asciifier;
use self::num::complex::{Complex64};


static HELP_TEXT: &'static str = include_str!("res/help.txt");


/**
 * Keeps a buffer which is a Matrix of chars, and prints a screenful to text to stdout
 */
pub struct TextBuffer<'a> {
    pub buffer: Matrix<char>,
   	help_text: Vec<&'a str>,
}

impl<'a> TextBuffer<'a> {

    pub fn new(width: usize, height: usize) -> TextBuffer<'a> {
        TextBuffer {
            buffer: Matrix::new(width, height),
   			help_text: HELP_TEXT.lines().collect(),
        }
    }

	pub fn set_size(&mut self, w: usize, h: usize) {
		self.buffer = Matrix::new(w, h);
	}

	/**
	 * Takes in position vals as i32 so that negative values can be handled
	 */
	pub fn draw_string(&mut self, string: &String, mut x: i32, y: i32) {
		if y < 0 || y >= self.buffer.height() as i32 {
			return;
		}
		for char in string.chars() {
			if x < 0 {
				x+= 1;
				continue;
			}
			if x >= self.buffer.width() as i32 {
				break;	
			}
			self.buffer.set(x as usize, y as usize, char);
			x += 1;
		}
	}

	/**
	 * Draw help text dialog
	 *
	 * offset_ratio: @0, dialog is fully visible; @1, dialog is off-screen
	 *
	 * TODO: make more general draw-block-of-text function
	 */
	pub fn draw_help_dialog(&mut self, 
			offset_ratio: f64, vp_pos: &Vector2f, zoom: f64, julia_c: Option<Complex64> ) {

		let help_text = self.help_text.clone();  // work around teh compiler :( ?!
		
		let mut y: i32 = (self.buffer.height() as i32 - self.help_text.len() as i32) / 2;  // vertically centered
		let mut x: i32 = self.buffer.width() as i32 - help_text[0].len() as i32;  // right-justified
		x += (help_text[0].len() as f64 * offset_ratio) as i32;

		for s in help_text {
			self.draw_string(&s.to_string(), x, y);
			y += 1;
		}

		// draw view specs
		let mut s;
		y = y - 6;
		
		// julia seed val
		match julia_c {
			Some(c) => {
				let s1 = format!("{:.*}", 8, c.re);
				let s2 = format!("{:.*}i", 8, c.im);
				s = format!("   c: {} {}", s1, s2);
				self.draw_string(&s, (x + 2), y);
			},
			None => { }
		}
		
		// position x 
		s = format!("{:.*}", 17, vp_pos.x);  // f64 can has 17 sig digits  
		if vp_pos.x >= 0.0 {
			s = "   x: +".to_string() + &s;
		} else {
			s = "   x: ".to_string() + &s;
		}
		self.draw_string(&s, (x + 2), (y + 1));

		// position y
		let mut s = format!("{:.*}", 17, vp_pos.y);
		if vp_pos.y >= 0.0 {
			s = "   y: +".to_string() + &s;
		} else {
			s = "   y: ".to_string() + &s;
		}
		self.draw_string(&s, (x + 2), (y + 2));
		
		// zoom
		s = format!("zoom:  {:.0}x", zoom);
		self.draw_string(&s, (x + 2), (y + 3));
	}

	/**
	 * Prints a screenful of text using the buffer data, in one single pass
	 */
    pub fn print(&self) {
    	for y in 0..self.buffer.height() {
    		let row = self.buffer.get_row(y);
    		let s: String = row.into_iter().cloned().collect();
    		print!("{}{}", ansi::move_cursor(0, y as i32), s);
    	}
    }
}
