// ansi escape sequences
// see http://academic.evergreen.edu/projects/biophysics/technotes/program/ansi_esc.htm
pub static CLEAR: &'static str = "\x1b[2J\x1b[H";
pub static TOP_LEFT: &'static str = "\x1b[H";

/**
 * 
 */
pub fn move_cursor(col:i32, row:i32) -> String {
	// rem, ansi row and col are 1-indexed
	format!("\x1b[{row};{col}H", row = row + 1, col = col + 1)
}
