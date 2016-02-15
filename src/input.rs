use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use rustbox;
use rustbox::{RustBox, Key, Mouse};
use rustbox::Event::{KeyEvent, MouseEvent};


// TODO: make this an Optional i think
#[derive(Debug)]
pub enum Command {
    PositionVelocity(f64,f64),
    PositionTween(i32, i32),
    Zoom(f64),
    ZoomContinuous(f64),
    Stop, Reset,
    Quit, 
    None  
}


#[allow(dead_code)]
pub struct ThreadLooper {
	fpo: u8
}

impl ThreadLooper {
	
	/**
	 * This spawns a thread which loops, polling for keyboard and mouse input us====ing rustbox.
	 * (Rustbox is only used for this purpose, not for any terminal output).
	 * 
	 * Note how data is not passed using a channel's sender, but by mutating the passed-in argument, 
	 * which is shared with the main thread. This value acts as a flag.
	 *
	 * TODO: Should not be a flag so much as a queue...
	 */
	pub fn go(wrapped_command: Arc<Mutex<Command>>) -> thread::JoinHandle<()> {
	
	    thread::spawn(move || {
	
		    let rustbox = match RustBox::init(
		    		rustbox::InitOptions { input_mode: rustbox::InputMode::EscMouse, buffer_stderr: false }) {  
		        Result::Ok(v) => v,
		        Result::Err(e) => panic!("{}", e),  // TODO: way to know about panic from outside thread
		    };
		
		    loop {

				let event = rustbox.poll_event(false);  // rem, this BLOCKS
		        
		        // TODO: use this instead, and rip out the thread nonsense
		        // let event = rustbox.peek_event(Duration::from_millis(5000), false); 
		        
		        match event {

		            Ok(KeyEvent(key)) => {
						
						let mut locked_command = wrapped_command.lock().unwrap();
		                match key {
		                    
		                    Key::Left => 
		                    	*locked_command = Command::PositionVelocity(-1.0, 0.0),
		                    Key::Right => 
		                    	*locked_command = Command::PositionVelocity(1.0, 0.0),
		                    Key::Up => 
		                    	*locked_command = Command::PositionVelocity(0.0, -1.0),
		                    Key::Down => 
		                    	*locked_command = Command::PositionVelocity(0.0, 1.0),
		                    
		                    Key::Char('a') | Key::Char('=') => 
		                    	*locked_command = Command::Zoom(-1.0),
		                    Key::Char('A') | Key::Char('+')=> 
		                    	*locked_command = Command::ZoomContinuous(-0.5),
		                    Key::Char('z') | Key::Char('-')=> 
		                    	*locked_command = Command::Zoom(1.0),
							Key::Char('Z') | Key::Char('_') => 
								*locked_command = Command::ZoomContinuous(0.5),
		                    
		                    Key::Char(' ') => 
		                    	*locked_command = Command::Stop,
		                    Key::Char('r') => 
		                    	*locked_command = Command::Reset,
		                    
		                    Key::Esc | Key::Ctrl('c') => { 
								*locked_command = Command::Quit;
								break;  // breaks out of loop, terminating thread  
		                    },	
		                    _ => {}
		                }

		            },

		            Ok(MouseEvent(mouse, x, y)) => {
   						
   						let mut locked_command = wrapped_command.lock().unwrap();
						match mouse {

							Mouse::WheelUp => 
								*locked_command = Command::Zoom(-0.3),
							Mouse::WheelDown => 
								*locked_command = Command::Zoom(0.3),
							Mouse::Left => 
								*locked_command = Command::PositionTween(x, y),
								
							_ => {},
						}                
		            },
		
		            Err(e) => panic!("{:?}", e),
		            
		            _ => { }
		        }
		    }
	    })
	}	
}
