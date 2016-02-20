// points of interest

static POI_TEXT: &'static str = include_str!("pois.txt");


pub type Poi = (f64, f64, f64);


pub struct Pois {
	pois: [Poi; 10]  // ie, one element per number key
}


impl Pois {
	pub fn new() -> Pois {
		let pois = Pois::parse_text_file();
		
		Pois { pois: pois }
	}	
	
	pub fn get(&self, index:usize) -> Poi {
		return self.pois[index].clone();
	}
	
	pub fn set( &self, index:usize, poi: Poi ) {
		// TODO
	}
	
	pub fn clear(index: usize) {
		// TODO
	}
	
	fn save() {
		// TODO?
	}
	
	//
	
	fn parse_text_file() -> [Poi; 10] {
		
		let mut pois = [(0.0, 0.0, 1.0); 10];
		let lines: Vec<&str> = POI_TEXT.lines().collect();
		let mut count = 0;
		for str in lines {
			let coords = Pois::parse_line(&str);
			match coords {
				Some(val) => {
					pois[count] = val;
					count += 1;
					if count >= 10 {
						break;
					}
				},
				_ => { }
			}
		}
		pois
	}
	
	fn parse_line(s: &str) -> Option<Poi> {
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
