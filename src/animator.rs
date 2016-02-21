use std::ops::{Add, Mul};
use vector2::Vector2f;


/**
 * Used to animate a value thru successive calls to "update()"
 *
 * TODO: Try to have Animator take a reference to the value
 *       rather than owning it, which limits its usefulness
 * TODO: Is currently in general super-unergonomic :(
 */
#[derive(Debug)]
pub struct Animator<T> where T:Add + Copy, T::Output:Add+Copy{
	pub value: T,
	spec: Spec<T>,
	target_is_gt: bool, // used for threshhold/completion test for Spec::Target
}

impl Animator<f64> {

	// Always use this
	// TODO: how to disallow the creation of a 'struct literal'?
	pub fn new(value: f64, spec: Spec<f64>) -> Animator<f64> {
		let mut a = Animator { value: value, spec: Spec::None, target_is_gt: false };
		a.set_spec(spec);
		a
	}

	pub fn spec(&self) -> &Spec<f64> {
		&self.spec
	}

	pub fn set_spec(&mut self, spec: Spec<f64>) {
		match spec {
			Spec::None => {}
			Spec::Target { ref target, .. } => {
				self.target_is_gt = target > &self.value;
			},
			_ => {}
		}
		self.spec = spec;
	}
	
	pub fn update(&mut self) {   
		
		let mut should_set_spec_none = false;
		
		match &mut self.spec {

			&mut Spec::Velocity { ref mut velocity, friction } => {
				self.value = self.value + *velocity;
				*velocity = *velocity * friction;
			},

			&mut Spec::VelocityWithRotation { .. } => {
				// not applicable
			}

			&mut Spec::Scale { ref mut scale, friction } => {
				self.value = self.value + (self.value *  *scale);
				*scale = *scale * friction;
			},

			&mut Spec::Target { target, coefficient, epsilon } => { 
				
				self.value = self.value + (target - self.value) * coefficient;
				
				match epsilon {
					Some(eps) => {
						if self.target_is_gt {
							let thresh = target - eps;
							if self.value >= thresh {
								// value is increasing and is almost at (or past) target
								self.value = target;
								should_set_spec_none = true;
							}
						} else {
							let thresh = target + eps;
							if self.value <= thresh {
								// value is decreasing and is almost at (or past) target
								self.value = target;
								should_set_spec_none = true;
							}
						}
					},
					_ => {}
				}
			},
			
			_ => {}
		}
		
		if should_set_spec_none {
			self.spec = Spec::None;
		}
	}
}

impl Animator<Vector2f> {
	
	pub fn new(value: Vector2f, spec: Spec<Vector2f>) -> Animator<Vector2f> {
		let mut a = Animator { value: value, spec: Spec::None, target_is_gt: false };
		a.set_spec(spec);
		a
	}

	pub fn spec(&mut self) -> &Spec<Vector2f> {
		&mut self.spec
	}

	pub fn set_spec(&mut self, spec: Spec<Vector2f>) {
		self.spec = spec;
	}

	pub fn update(&mut self) {
		   
		let mut should_set_spec_none = false;
		   
		match &mut self.spec {

			&mut Spec::Velocity { ref mut velocity, friction } => {
				self.value = self.value + *velocity;
				*velocity = *velocity * friction;
			},

			&mut Spec::VelocityWithRotation { ref mut velocity, rotation, friction } => {
				let vel = Vector2f::rotate(*velocity, rotation);
				self.value = self.value + vel;
				*velocity = *velocity * friction;
			},

			&mut Spec::Target { ref target, coefficient, epsilon } => {  
				self.value.x = self.value.x + (target.x - self.value.x) * coefficient;
				self.value.y = self.value.y + (target.y - self.value.y) * coefficient;
				
				match epsilon {
					// TODO: untested
					Some(eps) => {
						let v = Vector2f::new(self.value.x - target.x, self.value.y - target.y);
						if Vector2f::len(v) <= eps {
							self.value = *target;
							should_set_spec_none = true;
						}
					},
					_ => {}
				}
			},

			_ => {}
		}
		
		if should_set_spec_none {
			self.spec = Spec::None;
		}
	}
}

/**
 * Is the specification of how the value will be animated
 */
#[derive(Debug)]
pub enum Spec<T> where T:Add + Copy, T::Output:Add+Copy {

	// 'velocity' gets added to value; magnitude decays using 'friction'
	Velocity { velocity: T, friction: f64 },  

	// rotated 'velocity' gets added to value; magnitude decays using 'friction'
	VelocityWithRotation { velocity: T, rotation: f64, friction: f64 },  

	// 'value' is modified by 'scale'; 'scale' decays towards 1.0 using 'friction'
	// 'scale' would typically be a value very close to 1.0; think "scale velocity"
	Scale { scale: f64, friction: f64 },        

	// Value moves towards target (ease-out tween)
	// 
	// "epsilon" is the minimum distance from "target" at which the tween will be considered finished
	// (xeno's paradox kind of deal) 
	Target { target: T, coefficient: f64, epsilon: Option<f64>},
	
	None
	
	// TODO: need a type for Penner/Tweener style tweens  
}

impl<T> Spec<T> where T:Add + Copy, T::Output:Add+Copy {
	
	// TODO: this seems inelegant. what would be better?
	pub fn set_target(&mut self, new_target: T) {
		match self {
			&mut Spec::Target { ref mut target, .. } => {
				*target = new_target;
			},
			_ => panic!("Variant must be Spec::Target")
		}
	}
	
}
