use std::ops::{Add, Mul};
use leelib::vector2::Vector2f;


/**
 * Used to animate a value thru successive calls to "update()"
 *
 * TODO: Try to have Animator take a reference to the value
 *       rather than owning it, which limits its usefulness
 * TODO: Is currently in general super-unergonomic :(
 *       Probably should try trait'ed structs rather than enum for Anim
 * TODO: Add Penner-like params for Anim::Target
 */
#[derive(Debug)]
pub struct Animator<T> where T:Add + Copy, T::Output:Add+Copy {
	pub value: T,
	anim: Anim<T>,
	target_is_gt: bool, // used for threshhold/completion test for Anim::Target
}

impl<T> Animator<T> where T:Add + Copy, T::Output:Add+Copy {

	// set the target value of a Anim::Target
	pub fn set_target(&mut self, targ: T) {
		match self.anim {
			Anim::Target { ref mut target, .. } => {
				*target = targ;
			},
			_ => { panic!("Wrong variant for Anim"); }
		}
	}

	// Convenience functions for setting data in specific Anim variants	

	// set the rotation value of an Anim::Velocity
	pub fn set_velocity(&mut self, vel: T) {
		match self.anim {
			Anim::Velocity { ref mut velocity, .. } => {
				*velocity = vel;
			},
			_ => { panic!("Wrong variant for Anim"); }
		}
	}

	// set the velocity value of an Anim::VelocityWithRotation
	pub fn set_vwr_velocity(&mut self, vel: T) {
		match self.anim {
			Anim::VelocityWithRotation { ref mut velocity, .. } => {
				*velocity = vel;
			},
			_ => { panic!("Wrong variant for Anim"); }
		}
	}

	// set the rotation value of an Anim::VelocityWithRotation
	pub fn set_vwr_rotation(&mut self, rot: f64) {
		match self.anim {
			Anim::VelocityWithRotation { ref mut rotation, .. } => {
				*rotation = rot;
			},
			_ => { panic!("Wrong variant for Anim"); }
		}
	}
	
	// set the scalevelocity value of an Anim::ScaleVelocity
	pub fn set_scale_velocity(&mut self, vel: f64) {
		match self.anim {
			Anim::ScaleVelocity { ref mut scale_velocity, .. } => {
				*scale_velocity = vel;
			},
			_ => { panic!("Wrong variant for Anim"); }
		}
	}

}

impl Animator<f64> {

	// Always use this
	// TODO: how to disallow the creation of a 'struct literal'?
	pub fn new(value: f64, anim: Anim<f64>) -> Animator<f64> {
		let mut a = Animator { value: value, anim: Anim::None, target_is_gt: false };
		a.set_anim(anim);
		a
	}

	pub fn anim(&self) -> &Anim<f64> {
		&self.anim
	}

	pub fn set_anim(&mut self, anim: Anim<f64>) {
		match anim {
			Anim::None => {}
			Anim::Target { ref target, .. } => {
				self.target_is_gt = target > &self.value;
			},
			_ => {}
		}
		self.anim = anim;
	}
	
	pub fn update(&mut self) {   
		
		let mut should_set_anim_none = false;
		
		match &mut self.anim {

			&mut Anim::Velocity { ref mut velocity, friction, epsilon } => {

				self.value = self.value + *velocity;
				*velocity = *velocity * friction;

				match epsilon {
					Some(eps) => {
						if velocity.abs() < eps {
							should_set_anim_none = true;
						}
					},
					_ => { }
				}
			},

			&mut Anim::VelocityWithRotation { .. } => {
				// not applicable
			}

			&mut Anim::ScaleVelocity { ref mut scale_velocity, friction, epsilon } => {
				
				self.value = self.value + (self.value *  *scale_velocity);
				*scale_velocity = *scale_velocity * friction;
				
				match epsilon {
					Some(eps) => {
						if scale_velocity.abs() < eps {
							should_set_anim_none = true;
						}
					},
					_ => { }
				}
			},

			&mut Anim::Target { target, coefficient, epsilon } => { 
				
				self.value = self.value + (target - self.value) * coefficient;
				
				match epsilon {
					Some(eps) => {
						if self.target_is_gt {
							let thresh = target - eps;
							if self.value >= thresh {
								// value is increasing and is almost at (or past) target
								self.value = target;
								should_set_anim_none = true;
							}
						} else {
							let thresh = target + eps;
							if self.value <= thresh {
								// value is decreasing and is almost at (or past) target
								self.value = target;
								should_set_anim_none = true;
							}
						}
					},
					_ => {}
				}
			},
			
			_ => {}
		}
		
		if should_set_anim_none {
			self.anim = Anim::None;
		}
	}
}

impl Animator<Vector2f> {
	
	pub fn new(value: Vector2f, anim: Anim<Vector2f>) -> Animator<Vector2f> {
		let mut a = Animator { value: value, anim: Anim::None, target_is_gt: false };
		a.set_anim(anim);
		a
	}

	pub fn anim(&mut self) -> &Anim<Vector2f> {
		&mut self.anim
	}

	pub fn set_anim(&mut self, anim: Anim<Vector2f>) {
		self.anim = anim;
	}

	pub fn update(&mut self) {
		   
		let mut should_set_anim_none = false;
		   
		match &mut self.anim {

			&mut Anim::Velocity { ref mut velocity, friction, epsilon } => {
				
				self.value = self.value + *velocity;
				*velocity = *velocity * friction;
				
				match epsilon {
					Some(eps) => {
						if velocity.len() < eps {
							should_set_anim_none = true;
						}
					},
					_ => { }
				}
			},

			&mut Anim::VelocityWithRotation { ref mut velocity, rotation, friction } => {
				let vel = Vector2f::rotate(*velocity, rotation);
				self.value = self.value + vel;
				*velocity = *velocity * friction;
			},

			&mut Anim::Target { ref target, coefficient, epsilon } => {  
				self.value.x = self.value.x + (target.x - self.value.x) * coefficient;
				self.value.y = self.value.y + (target.y - self.value.y) * coefficient;
				
				match epsilon {
					// TODO: untested
					Some(eps) => {
						let v = Vector2f::new(self.value.x - target.x, self.value.y - target.y);
						if v.len() <= eps {
							self.value = *target;
							should_set_anim_none = true;
						}
					},
					_ => {}
				}
			},

			_ => {}
		}
		
		if should_set_anim_none {
			self.anim = Anim::None;
		}
	}
}

/**
 * Is the specification of how the value will be animated
 */
#[derive(Debug)]
pub enum Anim<T> where T:Add + Copy, T::Output:Add+Copy {

	// 'velocity' gets added to value; magnitude decays using 'friction'
	Velocity { velocity: T, friction: f64, epsilon: Option<f64> },  

	// rotated 'velocity' gets added to value; magnitude decays using 'friction';
	// rotation itself does not get animated
	// TODO: add 'epsilon' to be consistent
	VelocityWithRotation { velocity: T, rotation: f64, friction: f64 },  

	// 'value' is multiplied by 1.0 + 'scale'
	// TODO: scale_velocity can probably be of type T; then test with Vec2        
	ScaleVelocity { scale_velocity: f64, friction: f64, epsilon: Option<f64> },

	// Value moves towards target (ease-out tween)
	// "epsilon" is the minimum distance from "target" at which the tween will be considered finished
	// (xeno's paradox kind of deal) 
	Target { target: T, coefficient: f64, epsilon: Option<f64>},
	
	None
}
