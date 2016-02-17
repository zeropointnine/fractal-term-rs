use std::cmp::{max, min};
use ansi;
use matrix::Matrix;
use vector2::Vector2f;


/**
 * Prints a screenful of text to stdout using a buffer
 */
pub struct TextRenderer {
    buffer: Matrix<char>
}

impl TextRenderer {

    pub fn new(width: usize, height: usize) -> TextRenderer {
        TextRenderer {
            buffer: Matrix::new(width, height)
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
				let char = ascii.value_to_char(values.get(x, y) as f64);
				self.buffer.set(x, y, char);
			}
		}		
	}
	
	/**
	 *
	 */
	pub fn draw_string(&mut self, string: &String, mut x: usize, y: usize) {
		for char in string.chars() {
			self.buffer.set(x, y, char);
			x += 1;
		}
	}

	/**
	 *
	 */
	pub fn draw_text(&mut self, text: &Vec<&str>, pos: &Vector2f) {
		let mut y = (self.buffer.height() - text.len()) / 2;  // vertically centered
		let mut x = 0;
		for s in text {
			x = self.buffer.width() - s.len();  // right-justified
			self.draw_string(&s.to_string(), x, y);
			y += 1;
		}
		
		// TODO: this doesn't belong here:
		let mut s = format!("{:.*}", 17, pos.x);
		if pos.x >= 0.0 {
			s = "x  ".to_string() + &s;
		} else {
			s = "x ".to_string() + &s;
		}
		self.draw_string(&s, x + 4, y - 3);
		let mut s = format!("{:.*}", 17, pos.y);
		if pos.y >= 0.0 {
			s = "y  ".to_string() + &s;
		} else {
			s = "y ".to_string() + &s;
		}
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

// ---

const DEFAULT_CHARS: &'static str = " .,:;+*=ixcaoelf?IGUOQ08%X&#@";

/**
 * 'Asciifies' values into chars.
 * (isn't limited to ascii charset, of course)
 */
pub struct Asciifier {
    chars: Vec<char>,  // a collection of characters that are ordered by visual 'character weight'
    value_ceil: f64,
    value_step: f64,
}

impl Asciifier {
    pub fn new(value_ceil: f64) -> Asciifier {
        let mut ascii = Asciifier {
            chars: DEFAULT_CHARS.chars().collect(),
            value_ceil: value_ceil,
            value_step: 0.0,
        };
        ascii.update_value_step();
        ascii
    }

    pub fn set_chars(&mut self, charset: &String) {
        self.chars = charset.chars().collect();
        self.update_value_step();
    }

    pub fn set_value_ceil(&mut self, value_ceil: f64) {
        self.value_ceil = value_ceil;
        self.update_value_step();
    }
    
    pub fn value_to_char(&self, value: f64) -> char {
    	// TODO: can this be 'parameterized' with a function pointer?
    	if true {
    		self.transform_sqrt(value)
    	} else {
    		self.transform_linear(value)
    	}
    }
    
    pub fn transform_sqrt(&self, value: f64) -> char {
    	
    	let ratio = value / self.value_ceil;
    	let value = ratio.sqrt();
    	let value = value * self.value_ceil;
        self.transform_linear(value)
    }

    pub fn transform_linear(&self, value: f64) -> char {
        let mut i = (value / self.value_step) as usize;
        if i > self.chars.len() - 1 {
            i = self.chars.len() - 1;
        }
        self.chars[i]
    }

    fn update_value_step(&mut self) {
        self.value_step = self.value_ceil as f64 / self.chars.len() as f64;
    }
}
