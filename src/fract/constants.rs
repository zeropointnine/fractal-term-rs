use std;

// a given terminal may not (probably won't) show every frame @ 60fps, but
pub const TARGET_FPS: i32 = 60;   

// rough estimate of terminal character a/r, which we can't rly know
pub const CHARACTER_ASPECT_RATIO: f64 = 0.4;

pub static MANDELBROT_POI_TEXT: &'static str = include_str!("res/mandelbrot_pois.txt");
pub static JULIA_COMPLEX_TEXT: &'static str = include_str!("res/julia_complex.txt");

pub const DEG: f64 = std::f64::consts::PI / 180.0;

// movement-related values
pub const ZOOM_INCREMENT: f64 = 0.015;
pub const VELOCITY_RATIO_INCREMENT: f64 = 0.007;
pub const ROTATIONAL_VELOCITY_INCREMENT: f64 = 1.2 * DEG;
pub const TARGET_COEF: f64 = 0.08;
pub const FRICTION: f64 = 0.95;

pub const SHOW_DEBUG_TEXT: bool = false;
