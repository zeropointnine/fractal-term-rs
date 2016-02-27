use math;

// simple version
// const DEFAULT_CHARS: &'static str = " .,:;i1tfLCG08@";

// calibrated vaguely for Monaco 12
const DEFAULT_CHARS: &'static str = " .,`'\"^:;-~=+*ixcnaeomlfh1IEUOQWX%#$&@";
 

/**
 * 'Asciifies' values into chars.
 * (isn't limited to ascii charset, of course)
 */
pub struct Asciifier {
    chars: Vec<char>,  // a collection of characters that are ordered by visual 'character weight'
    floor: f64,
    ceil: f64,
    bias: f64,
    
    range: f64,
    step: f64,
}

impl Asciifier {
    pub fn new(floor: f64, ceil: f64) -> Asciifier {
        let mut ascii = Asciifier {
            chars: DEFAULT_CHARS.chars().collect(),
            floor: floor,
            ceil: ceil,
            bias: 0.0,
            
            range:0.0,
            step: 0.0,
        };
        ascii.update();
        ascii
    }

    pub fn set_chars(&mut self, charset: &String) {
        self.chars = charset.chars().collect();
        self.update();
    }

	pub fn floor(&self) -> f64 {
		self.floor
	}

	pub fn ceil(&self) -> f64 {
		self.ceil
	}

	/**
	 * Typical use would be to set floor to 0  
	 * and set ceil to whatever the max value is of the data set
	 */
    pub fn set_range(&mut self, floor: f64, ceil: f64) {
        self.floor = floor;
        self.ceil = ceil;
        self.update();
    }
    
    pub fn bias(&self) -> f64 {
    	self.bias
    }
    pub fn set_bias(&mut self, bias: f64) {
    	self.bias = bias;
    }

    pub fn to_char(&self, mut value: f64) -> char {
    	
    	if value < self.floor {
    		value = self.floor;
    	} else if value > self.ceil {
    		value = self.ceil;
    	}
    	
    	let ratio = (value - self.floor) / self.range;
    	
    	let biased_a = ratio.sqrt();    
    	let biased_b = ratio * ratio;  
    	
    	let ratio = math::map(self.bias, -1.0, 1.0,  biased_a, biased_b);
    	
    	// less hump:
		// biased_a = (((ratio * 3.0) + 1.0) as f64).ln() * (5.0/7.0);
		// even less:
		// ratio_b = ((ratio + 1.0) as f64).ln() * (10.0/7.0);
		
    	let mut i = (ratio / self.step) as usize;
        if i > self.chars.len() - 1 {
            i = self.chars.len() - 1;
        }
        self.chars[i]
    }
    
	/**
	 * Given a char set, a ceiling value and a floor value,
	 * cache range and step values
	 */ 
    fn update(&mut self) {
    	self.range = self.ceil - self.floor;
        self.step = 1.0 / self.chars.len() as f64;
    }
}
