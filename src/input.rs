use std::thread;
use std::sync::{Arc, Mutex};
use rustbox;
use rustbox::{RustBox, Key, Mouse, EventResult};
use rustbox::Event::{KeyEvent, MouseEvent, ResizeEvent};


/**
 * Spawns a thread which loops, polling for keyboard and mouse input using rustbox.
 * (Rustbox is only used for this purpose, not for any terminal output).
 * 
 * Note how data is not passed using a channel's sender, but by mutating the passed-in argument, 
 * which is shared with the main thread. This value acts as a flag.
 *
 * TODO: Should not be a flag so much as a queue... (eg, fast mousewheel operations lag)
 */
pub fn launch_thread(wrapped_command: Arc<Mutex<Command>>) -> thread::JoinHandle<()> {

    thread::spawn(move || {

	    let rustbox = match RustBox::init(
	    		rustbox::InitOptions { input_mode: rustbox::InputMode::EscMouse, buffer_stderr: false }) {  
	        Result::Ok(v) => v,
	        Result::Err(e) => panic!("{}", e),
	    };

		{
			// immediately set the command to tell app the terminal's character dimensions
			let mut locked_command = wrapped_command.lock().unwrap();
			*locked_command = Command::Size(rustbox.width(), rustbox.height());
		}	    		
	
	    loop {
 
			let event = rustbox.poll_event(false);  // rem, this BLOCKS
	        // TODO: use this instead, and rip out the thread nonsense
	        // let event = rustbox.peek_event(Duration::from_millis(5000), false); 
	        
			let mut locked_command = wrapped_command.lock().unwrap();
			*locked_command = Command::from_rustbox_event(event);

	        match *locked_command {
	        	Command::Quit => {
	        		break;
	        	}
	        	_ => {} 
	        }
	    }
    })
}	


#[derive(Debug)]
pub enum Command {
    PositionVelocity(f64,f64),
    PositionTween(i32, i32),
    Zoom(f64),
    ZoomContinuous(f64),
    RotationVelocity(f64),
    Size(usize, usize),
    Poi(usize),
    AutoExposure, Help, Stop, Reset, Quit, 
    None, 
    // TODO: use 'Option' pattern instead of 'none' ?  
}

impl Command {

	pub fn from_rustbox_event(event_result: EventResult) -> Command {

		let event = event_result.unwrap();
        match event {
        
            KeyEvent(key) => {				
                
                match key {
                	
                    Key::Left => Command::PositionVelocity(-1.0, 0.0),
                    Key::Right => Command::PositionVelocity(1.0, 0.0),
                    Key::Up => Command::PositionVelocity(0.0, -1.0),
                    Key::Down => Command::PositionVelocity(0.0, 1.0),
                    
                    Key::Char('a') | Key::Char('=') => Command::Zoom(-1.0),
                    Key::Char('A') | Key::Char('+') => Command::ZoomContinuous(-0.5),
                    Key::Char('z') | Key::Char('-') => Command::Zoom(1.0),
					Key::Char('Z') | Key::Char('_') => Command::ZoomContinuous(0.5),
                    
                    Key::Char('[') | Key::Char('{') => Command::RotationVelocity(1.0),
                    Key::Char(']') | Key::Char('}') => Command::RotationVelocity(-1.0),

                    Key::Char('/') | Key::Char('?') | Key::Char('h') | Key::Char('H') => Command::Help,
                    
                    Key::Char('1') => Command::Poi(0),
                    Key::Char('2') => Command::Poi(1),
                    Key::Char('3') => Command::Poi(2),
                    Key::Char('4') => Command::Poi(3),
                    Key::Char('5') => Command::Poi(4),
                    Key::Char('6') => Command::Poi(5),
                    Key::Char('7') => Command::Poi(6),
                    Key::Char('8') => Command::Poi(7),
                    Key::Char('9') => Command::Poi(8),
                    Key::Char('0') => Command::Poi(9),
                    
                    Key::Char('e') | Key::Char('E') => Command::AutoExposure,
                    Key::Char(' ') => Command::Stop,
                    Key::Char('r') => Command::Reset,
                    Key::Esc | Key::Ctrl('c') => Command::Quit,
                    
                    _ => Command::None,
	            }
            }

		    MouseEvent(mouse, x, y) => {
				match mouse {
					Mouse::WheelUp => Command::Zoom(-0.3),
					Mouse::WheelDown => Command::Zoom(0.3),
					Mouse::Left => Command::PositionTween(x, y),
					_ => Command::None
					
				}                
	        },
		    
		    ResizeEvent(w, h) => {
		    	Command::Size(w as usize, h as usize)
		    },
		    
   		    _ => {
   		    	Command::None
   		    }
        }
	}
}
