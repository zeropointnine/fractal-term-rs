use fract::view::View;

/**
 * Simple vector wrapper of boxed Views 
 */
pub struct Views {
	pub vec: Vec<Box<View>>,
	pub index: usize,
}

impl Views {
	
	pub fn new() -> Self {
		Views { vec: Vec::new(), index:0 }
	}
	
	pub fn get(&mut self) -> &mut View {
		&mut (*self.vec[self.index])
	}
	pub fn get_im(&self) -> &View {
		&(*self.vec[self.index])
	}
	
	pub fn get_num(&mut self, i: usize) -> &mut View {
		&mut (*self.vec[i])
	}
	pub fn get_num_im(&self, i: usize) -> &View {
		&(*self.vec[i])
	}
}
