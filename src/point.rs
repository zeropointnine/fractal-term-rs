#[derive(Debug, Clone)]
pub struct Point {
	pub x: f64,
	pub y: f64
}

impl Point {
	pub fn len(&self) -> f64 {
		(self.x * self.x + self.y * self.y).sqrt() 
	}
	pub fn multiply_by(&mut self, val: f64) {
		self.x = self.x * val;
		self.y = self.y * val;
	}
	pub fn add_by(&mut self, pt: &Point) {
		self.x = self.x + pt.x;
		self.y = self.y + pt.y;
	}
}

