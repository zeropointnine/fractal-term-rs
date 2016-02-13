use std::ops::{Add, Mul};
use vector2::Vector2f;


#[derive(Debug)]

/**
 * Animates a float or Vector2 thru successive calls to "update()"
 */
pub struct Animator<T> where T:Add + Copy, T::Output:Add+Copy{
	pub value: T,
	pub spec: Spec<T> 
}

impl Animator<f64> {
	pub fn update(&mut self) {   
		match &mut self.spec {
			&mut Spec::Velocity { ref mut velocity, friction } => {
				self.value = self.value + *velocity;
				*velocity = *velocity * friction;
			},
			&mut Spec::Scale { ref mut scale, friction } => {
				self.value = self.value + (self.value *  *scale);
				*scale = *scale * friction;
			},
			&mut Spec::Tween { .. } => { 
				// TODO: implement
			},
			_ => {}
		}
	}
}

impl Animator<Vector2f> {
	pub fn update(&mut self) {   
		match &mut self.spec {
			&mut Spec::Velocity { ref mut velocity, friction } => {
				self.value = self.value + *velocity;
				*velocity = *velocity * friction;
			},
			&mut Spec::Tween { ref target, coefficient } => {  
				self.value.x = self.value.x + (target.x - self.value.x) * coefficient;
				self.value.y = self.value.y + (target.y - self.value.y) * coefficient;
			},
			_ => {}
		}
	}
}

/**
 * Describes how the value will be animated
 */
#[derive(Debug)]
pub enum Spec<T> where T:Add + Copy, T::Output:Add+Copy {

	// 'velocity' gets added to value; velocity decays using 'friction'
	Velocity { velocity: T, friction: f64 },  

	// 'value' is modified by 'scale'; 'scale' decays towards 1.0 using 'friction'
	// 'scale' would typically be a value very close to 1.0; think "scale velocity"
	Scale { scale: f64, friction: f64 },        

	// Value moves towards target (ease-out tween)
	// TODO: refactor to enable Penner style tweens 
	Tween { target: T, coefficient: f64},
	
	None
}
