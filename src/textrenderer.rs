use std::cmp::{max, min};
use ansi;
use matrix::Matrix;
use vector2::Vector2f;
use asciifier::Asciifier;


static HELP_TEXT: &'static str = include_str!("help.txt");


/**
 * Prints a screenful of text to stdout using a buffer
 */
pub struct TextRenderer<'a> {
    buffer: Matrix<char>,
   	help_text: Vec<&'a str>,
}

impl<'a> TextRenderer<'a> {

    pub fn new(width: usize, height: usize) -> TextRenderer<'a> {
        TextRenderer {
            buffer: Matrix::new(width, height),
   			help_text: HELP_TEXT.lines().collect(),
        }
    }

	pub fn size(&mut self, w: usize, h: usize) {
		self.buffer = Matrix::new(w, h);
	}

	/**
	 * 
	 */
	pub fn draw_ascii_rect(&mut self, values: &Matrix<u16>, ascii: &Asciifier) {
		let w = min(self.buffer.width(), values.width());
		let h = min(self.buffer.height(), values.height());
		for y in 0..h {
			for x in 0..w {
				let char = ascii.to_char(values.get(x, y) as f64);
				self.buffer.set(x, y, char);
			}
		}		
	}
	
	/**
	 *
	 */
	pub fn draw_string(&mut self, string: &String, mut x: usize, y: usize) {
		for char in string.chars() {
			if x >= self.buffer.width() {
				break;	
			}
			self.buffer.set(x, y, char);
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
	pub fn draw_help_dialog(&mut self, offset_ratio: f64, vp_pos: &Vector2f, zoom: f64) {

		// TODO: doing this temporarily to keep compiler from complaining, 
		//       but need to figure out correct way to handle this
		let help_text = self.help_text.clone();
		
		let mut y = (self.buffer.height() - self.help_text.len()) / 2;  // vertically centered
		let mut x = self.buffer.width() - help_text[0].len();  // right-justified
		x += (help_text[0].len() as f64 * offset_ratio) as usize;

		for s in help_text {
			self.draw_string(&s.to_string(), x, y);
			y += 1;
		}

		// draw viewport position; f64 should have up to 17 sig digits
		let mut s = format!("{:.*}", 17, vp_pos.x);  
		if vp_pos.x >= 0.0 {
			s = "x    ".to_string() + &s;
		} else {
			s = "x   ".to_string() + &s;
		}
		self.draw_string(&s, x + 4, y - 4);
		let mut s = format!("{:.*}", 17, vp_pos.y);
		if vp_pos.y >= 0.0 {
			s = "y    ".to_string() + &s;
		} else {
			s = "y   ".to_string() + &s;
		}
		self.draw_string(&s, x + 4, y - 3);
		
		s = format!("zoom {:.0}x", zoom);
		self.draw_string(&s, x + 4, y - 2);
	}

	/**
	 * Prints a screenful of text using the buffer data, in one single pass
	 * (minimizes/eliminates flicker)  
	 */
    pub fn render(&self) {
    	for y in 0..self.buffer.height() {
    		let row = self.buffer.get_row(y);
    		let s: String = row.into_iter().cloned().collect();
    		print!("{}{}", ansi::move_cursor(0, y as i32), s);
    	}
    }
}
