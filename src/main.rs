mod vector2;
mod math;
mod ansi;
mod animator;
mod textrender;
mod input;
mod nummap;
mod complex;
mod mandelbrot;
mod viewport;
mod app;

extern crate time;
extern crate rand;
extern crate rustbox;

use std::thread;
use std::sync::{Arc, Mutex};
use time::PreciseTime;
use input::Command;
use app::App;


const SHOW_DEBUG_INFO:bool = true;


fn main() {

    // debug-related 
    let mut frame_num = 0;
    let mut fps_timestamp = PreciseTime::now();
    let mut cum_calc_duration:i64 = 0;  
    let mut cum_render_duration:i64 = 0; 
    let mut frame_start_time:PreciseTime;
    let mut avg_fps:f64 = 0.0;
    let mut avg_calc_time:f64 = 0.0;
    let mut avg_render_time:f64 = 0.0;
    let mut message = "".to_string();

    let update_interval = 16_000i64;

	// use rustbox just long enough to get terminal dimensions
	let screen_width: u16;
	let screen_height: u16;
    {
    	let opts = rustbox::InitOptions { input_mode: rustbox::InputMode::AltMouse, buffer_stderr: false };
        let rustbox = match rustbox::RustBox::init(opts) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };
        screen_width = rustbox.width() as u16;
        screen_height = rustbox.height() as u16;
		// println!("dimensions: {}x{}", view_width, view_height);
    }

	let command = Command::None; 
    let wrapped_command = Arc::new(Mutex::new(command));
	// -----------------------------------------------------------    
	// START THE INPUT THREAD
	let handle = input::ThreadLooper::go(wrapped_command.clone());
	// -----------------------------------------------------------
    let wrapped_command = wrapped_command.clone();  // for use by main thread

	let mut app = App::new(screen_width, screen_height);

    loop {

        frame_start_time = PreciseTime::now();

		let mut should_refresh = false;
		if frame_num <= 1 {
			should_refresh = true;
		}

		{
			let mut locked_command = wrapped_command.lock().unwrap();
			{
				match *locked_command {
					Command::None => {},
					Command::Quit => break, // breaks out of loop to quit program	
					_ => {
						// input thread changed 'command', so handle it
						app.handle_command(&*locked_command);
						// reset command  
						*locked_command = Command::None;   	
					}
				}				
			}
		}
		
		app.update();
		
		if false &&  ! should_refresh {	
			thread::sleep_ms((update_interval / 1000) as u32);
			continue;
		}

		app.calculate();

        let calc_end_time = PreciseTime::now();

		app.render();			

        let render_end_time = PreciseTime::now();

        // debug info  
        let calc_time = frame_start_time.to(calc_end_time).num_microseconds().unwrap();
        cum_calc_duration = cum_calc_duration + calc_time;
        let render_time = calc_end_time.to(render_end_time).num_microseconds().unwrap();
        cum_render_duration = cum_render_duration + render_time;
        
        frame_num = frame_num + 1;
        if frame_num % 60 == 0 {
            // calc fps
            let usec = fps_timestamp.to(render_end_time).num_microseconds().unwrap();
            let usec_per_frame = usec as f64 / 60.0;
            avg_fps = 1.0 / (usec_per_frame as f64 / 1_000_000f64);
            avg_calc_time = cum_calc_duration as f64  / 60.0;
            avg_render_time = cum_render_duration as f64 / 60.0;

            // reset values
            fps_timestamp = PreciseTime::now();
            cum_calc_duration = 0;
            cum_render_duration = 0;
        }
        
        let mut s = format!("{} {:.0}x ", 
        	ansi::move_cursor((screen_height - 0) as i32, 1), app.get_magnification());
        
        if SHOW_DEBUG_INFO {
        	let s2 = format!("{:.2}fps {:.0}μs + {:.0}μs {} {}",
	            avg_fps, avg_calc_time, avg_render_time, frame_num, message);
        	s = s + &s2;  
        }
        print!("{}", s);

		/*        
        if SHOW_DEBUG_INFO {
        	let col =  (screen_width as i32) / 2  +  1;
        	let row = (screen_height as i32) / 2  +  1;
        	print!("{}█", ansi::move_cursor(row, col));
        }
        */

		// sleep until time for next frame	
		// TODO: sleep_ms is now deprecated
        let dur = frame_start_time.to(PreciseTime::now()).num_microseconds().unwrap();
        let mut sleep_time = update_interval - dur;  
        // hand-wavy adjust for unaccounted-for overhead
        sleep_time = sleep_time - 500;  
        let mut sleep_time_ms = sleep_time / 1000;
        if sleep_time_ms < 1 {
            sleep_time_ms = 1;
        }
        thread::sleep_ms(sleep_time_ms as u32);
	}
    
    // exit program 
    let _ = handle.join();
	print!("{}", ansi::CLEAR);
}
