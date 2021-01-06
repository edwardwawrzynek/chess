use std::fmt;
use std::ffi;
use std::fmt::Display;

mod clib;

/// A position on a chessboard
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BoardPos(clib::board_pos);

impl BoardPos {
    /// Construct a board position from an x and y
    pub fn new(x: i32, y: i32) -> BoardPos {
        BoardPos(unsafe { clib::board_pos_from_xy(x, y) })
    }

    /// Get the x value of the position
    pub fn x(self) -> i32 {
        unsafe { clib::board_pos_to_x(self.0) }
    }

    /// Get the y value of the position
    pub fn y(self) -> i32 {
        unsafe { clib::board_pos_to_y(self.0) }
    }

    /// An invalid board position (outside the chessboard)
    pub const INVALID: BoardPos = BoardPos(clib::BOARD_POS_INVALID as u8);
}

impl Display for BoardPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res: [i8; 3] = [0; 3];
        unsafe { clib::board_pos_to_str(self.0, res.as_mut_ptr()) };
        let c_str = unsafe { ffi::CStr::from_ptr(res.as_ptr()) };
        write!(f, "{}", c_str.to_str().unwrap())
    }
}

/// A structure with a bit coresponding to each square
pub struct Bitboard(clib::bitboard);

#[cfg(test)]
mod tests {
    use crate::BoardPos;
    #[test]
    fn it_works() {
        println!("{}", BoardPos::new(1, 1));
    }
}
