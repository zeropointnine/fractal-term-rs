pub struct DirtyChecker {
	vals: Vec<f64>,
	result: bool,
	force_flag: bool
}

impl DirtyChecker {
	
	pub fn new(num_vals: usize) -> Self {
		DirtyChecker { vals: vec!(0.0; num_vals), result: true, force_flag: false }
	}
	
	/**
	 * Compares incoming values against current values;
	 * saves them for the next check;
	 * stores the result in `result`, and returns it as well 
	 */ 
	pub fn do_check(&mut self, vals: Vec<f64>) -> bool {
		
		assert!(vals.len() == self.vals.len(), 
				format!("vector lengths must be consistent ({} vs {}", vals.len(), self.vals.len()));
		
		let mut b = false;
		for i in 0..self.vals.len() {
			if self.vals[i] != vals[i] {
				b = true;
			}
			self.vals[i] = vals[i];
		}
		self.result = b || self.force_flag;
		if self.force_flag {
			self.force_flag = false;
		}
		self.result
	}

	/**
	 * Sometimes you may want to set to `dirty` for other reasons
	 */
	pub fn force_dirty(&mut self) {
		self.force_flag = true;
	}
	
	pub fn is_dirty(&self) -> bool {
		self.result || self.force_flag
	}
}
