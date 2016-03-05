extern crate time;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use self::time::PreciseTime;
use leelib::ansi;
use fract::constants;
use fract::input;
use fract::input::Command;
use fract::App;


/**
 * Manages the main program loop, and hands off `Commands` from the user-input thread  
 */
pub fn main() {

	let command = Command::None; 
    let wrapped_command = Arc::new(Mutex::new(command));
	let handle = input::launch_thread(wrapped_command.clone());
    let wrapped_command = wrapped_command.clone();  // for use by main thread

	let mut timing = Timing::new(constants::TARGET_FPS);

	let mut app = App::new();

    loop {

        timing.frame_start();

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

		timing.calc_start();
		app.calculate();
        timing.calc_end();

		timing.draw_start();
		app.draw(&timing.averages_info);
		timing.draw_end();			

        thread::sleep(timing.get_sleep_duration());
	}
    
    // quit
    print!("{}", ansi::CLEAR); 
    let _ = handle.join();
} 

/**
 * Benchmarking logic for the main program loop 
 */
struct Timing {

	target_fps: i32,
    
    frame_start_time:PreciseTime,
    calc_start_time:PreciseTime,
    draw_start_time:PreciseTime,

    frame_num: i32,
    averages_start_time: PreciseTime,

    cum_calc_duration: i64,
    cum_draw_duration: i64, 
    avg_fps: f64,
    avg_calc_time: f64,
    avg_draw_time: f64,
    averages_info: String,
}

impl Timing {
	
	pub fn new(target_fps: i32) -> Timing {
		
		Timing {
			target_fps: target_fps,
			
			frame_start_time: PreciseTime::now(),
		    calc_start_time: PreciseTime::now(),
		    draw_start_time: PreciseTime::now(),

			frame_num: -1,
			averages_start_time: PreciseTime::now(),
			
			cum_calc_duration: 0,
			cum_draw_duration: 0,
			avg_fps: 0.0,
			avg_calc_time: 0.0,
			avg_draw_time: 0.0,
			averages_info: "".to_string(),
		}
	}
	
	pub fn frame_start(&mut self) {

		self.frame_start_time = PreciseTime::now();
		
        self.frame_num += 1;
        if self.frame_num % self.target_fps == 0 {
            let usec = self.averages_start_time.to(self.frame_start_time).num_microseconds().unwrap();
            let usec_per_frame = usec as f64 / self.target_fps as f64;
            self.avg_fps = 1.0 / (usec_per_frame as f64 / 1_000_000f64);
            self.avg_calc_time = self.cum_calc_duration as f64  / self.target_fps as f64;
            self.avg_draw_time = self.cum_draw_duration as f64 / self.target_fps as f64;
			self.averages_info = format!(" fps {:.2} calc {:.0}μs render {:.0}μs ", 
	    		self.avg_fps, self.avg_calc_time, self.avg_draw_time);

            // reset values
            self.averages_start_time = self.frame_start_time;
            self.cum_calc_duration = 0;
            self.cum_draw_duration = 0;
        }
	}
	
	pub fn calc_start(&mut self) {
		self.calc_start_time = PreciseTime::now();
	}
	pub fn calc_end(&mut self) {
		let dur = self.calc_start_time.to(PreciseTime::now()).num_microseconds().unwrap();
		self.cum_calc_duration += dur 
	}
	
	pub fn draw_start(&mut self) {
		self.draw_start_time = PreciseTime::now();
	}
	pub fn draw_end(&mut self) {
		let dur = self.draw_start_time.to(PreciseTime::now()).num_microseconds().unwrap();
		self.cum_draw_duration += dur 
	}
	
	/**
	 * Calculate the sleep duration needed for program loop to update at the target_fps
	 */
	pub fn get_sleep_duration(&self) -> Duration {
		
		let interval: i32 = (1_000_000_000 / self.target_fps) as i32;
        let elapsed: i32 = self.frame_start_time.to(PreciseTime::now()).num_nanoseconds().unwrap() as i32;
        let mut duration: i32 = interval - elapsed - 1_000_000;  // vague adjustment for unknown overhead  
        if duration < 0 {
            duration = 0;
        }
        return Duration::new(0, duration as u32);
	}
}
