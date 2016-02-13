// ansi escape sequences
// http://academic.evergreen.edu/projects/biophysics/technotes/program/ansi_esc.htm

pub static CLEAR: &'static str = "\x1b[2J\x1b[H";
pub static TOP_LEFT: &'static str = "\x1b[H";

/**
 * row and col are 1-indexed
 */
pub fn move_cursor(row:i32, col:i32) -> String {
	format!("\x1b[{row};{col}H", row = row, col = col)
}
