extern crate num;
use std::str::FromStr;
use self::num::complex::{Complex, Complex64};


pub type Three64 = (f64, f64, f64);


/**
 * Array of coordinate data that comes from a text file 
 */
pub struct Coords<T:Clone> {
	coords: [T; 10]  // ie, one element per number key
}


impl<T:Clone> Coords<T> {
	
	pub fn get(&self, index:usize) -> T {
		return self.coords[index].clone();
	}
	
	pub fn len(&self) -> usize {
		self.coords.len()
	}
	
	pub fn set( &self, index:usize, coord: T ) {
		// TODO
	}
	
	pub fn clear(index: usize) {
		// TODO
	}
	
	fn save() {
		// TODO?
	}
}

impl Coords<Three64> {

	pub fn new(textfile: &str) -> Coords<Three64> {
		Coords { coords: Coords::parse_pois(textfile) }
	}	

	fn parse_pois(textfile: &str) -> [Three64; 10] {
		
		let mut coords = [(0.0, 0.0, 1.0); 10];
		let lines: Vec<&str> = textfile.lines().collect();
		let mut count = 0;
		for str in lines {
			let poi = Coords::parse_line_poi(&str);
			match poi {
				Some(val) => {
					coords[count] = val;
					count += 1;
					if count >= 10 {
						break;
					}
				},
				None => { }
			}
		}
		coords
	}
	
	fn parse_line_poi(s: &str) -> Option<Three64> {
		let v: Vec<&str> = s.split(',').collect();
		if v.len() != 3 {
			return None
		}
		let res1 = v[0].trim().parse::<f64>();
		match res1 {
			Err(_) => return None,
			_ => {}
		}
		let res2 = v[1].trim().parse::<f64>();
		match res2 {
			Err(_) => return None,
			_ => {}
		}
		let res3 = v[2].trim().parse::<f64>();
		match res3 {
			Err(_) => return None,
			_ => {}
		}
		
		let x = res1.unwrap();
		let y = res2.unwrap();
		let zoom = res3.unwrap();
		Some( (x, y, zoom) )
	}
}

impl Coords<Complex64> {

	pub fn new(textfile: &str) -> Coords<Complex64> {
		Coords { coords: Coords::parse_complex(textfile) }
	}	

	fn parse_complex(textfile: &str) -> [Complex64; 10] {
		
		let mut coords = [Complex { re: 0.0, im: 0.0 }; 10];
		let lines: Vec<&str> = textfile.lines().collect();
		let mut count = 0;
		for str in lines {
			let poi = Coords::parse_line_complex(&str);
			match poi {
				Some(val) => {
					coords[count] = val;
					count += 1;
					if count >= 10 {
						break;
					}
				},
				None => { }
			}
		}
		coords
	}
	
	fn parse_line_complex(s: &str) -> Option<Complex64> {
		
		let v: Vec<&str> = s.split(',').collect();
		if v.len() != 2 {
			return None
		}
		
		let re;
		let result = v[0].trim().parse::<f64>();
		match result {
			Err(_) => return None,
			Ok(val) => re = val,
		}
		
		let mut im;
		let result = v[1].trim().parse::<f64>();
		match result {
			Err(_) => return None,
			Ok(val) => im = val,
		}
		
		Some( Complex { re: re, im: im } )
	}
}
