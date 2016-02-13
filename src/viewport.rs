use vector2::Vector2f;


pub struct Viewport {
	pub center: Vector2f,
	pub width: f64,
	pub max_width: f64,
}

impl Viewport {
	pub fn new(max_width: f64) -> Viewport {
		Viewport { 
			center: Vector2f { x: 0.0, y: 0.0 }, 
			width: max_width,
			max_width: max_width }
	}
}
