use std::cmp::{max, min};
use ansi;
use matrix::Matrix;

/**
 * Prints a screenful of text to stdout using a buffer
 */
pub struct TextPrinter {
    width: usize,
    height: usize,
    buffer: Matrix<char>
}

impl TextPrinter {

    pub fn new(width: usize, height: usize) -> TextPrinter {
        let m = Matrix::new(width, height);
        TextPrinter {
            width: width,
            height: height,
            buffer: m
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
	 * Prints a screenful of text using the buffer data, in one single pass
	 * (minimizes/eliminates flicker)  
	 */
    pub fn render(&self) {
    	for y in 0..self.buffer.height() {
    		let row = self.buffer.get_row(y);
    		let s: String = row.into_iter().cloned().collect();
    		print!("{}{}", ansi::move_cursor((y + 1) as i32, 1), s);
    	}
    }
}

// ---

// a collection of characters vaguely ordered by 'character weight'
const DEFAULT_CHARS: &'static str = ".,:;*icxeaoIGUOQ08%X#@";

/**
 * 'Asciifies' values into chars.
 * (isn't limited to ascii charset, of course)
 */
pub struct Asciifier {
    chars: Vec<char>,
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
