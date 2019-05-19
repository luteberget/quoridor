/// Get a move from a string on the form 
///
/// From wikipedia/quoriodor:
/// The notation proposed is similar to algebraic chess notation. Each square 
/// gets a unique letter-number designation. Columns are labeled a through i 
/// from player 1's left and rows are numbered 1 through 9 from player 2's 
/// side to player 1's side. A move is recorded as the column followed by the 
/// row as in e8. Player 1's pawn starts on e9 and player 2's pawn starts on e1.
///  
///  Each pawn move is defined by the new square occupied by the pawn. For 
///  example, if player 1 moves from square e9 to e8, player 1’s move is e8.
///  
///  Each fence move is defined by the square directly to the northwest of the 
///  wall center from player 1's perspective, as well as an orientation 
///  designation. For example: a vertical wall between columns e and f and 
///  spanning rows 3 and 4 would be given the designation e3v.
///  

use crate::*;

pub fn parse(s :&str) -> Result<Move,()> {
    let mut x = None;
    let mut y = None;
    let mut orientation = None;
    for c in s.chars() {
        if c.is_whitespace() { continue; }

        if x.is_none() {
            match c {
                'a' => { x = Some(1); },
                'b' => { x = Some(2); },
                'c' => { x = Some(3); },
                'd' => { x = Some(4); },
                'e' => { x = Some(5); },
                'f' => { x = Some(6); },
                'g' => { x = Some(7); },
                'h' => { x = Some(8); },
                'i' => { x = Some(9); },
                _ => { return Err(()); },
            };
        } else if y.is_none() {
            match c {
                '1' => { y = Some(1); },
                '2' => { y = Some(2); },
                '3' => { y = Some(3); },
                '4' => { y = Some(4); },
                '5' => { y = Some(5); },
                '6' => { y = Some(6); },
                '7' => { y = Some(7); },
                '8' => { y = Some(8); },
                '9' => { y = Some(9); },
                _ => { return Err(()); },
            };
        } else if orientation.is_none() {
            match c {
                'h' => { orientation = Some(Orientation::Horizontal); },
                'v' => { orientation = Some(Orientation::Vertical); },
                _ => { return Err(()); },
            };
        } else {
            return Err(());
        }
    }

    if let Some(x) = x {
        if let Some(y) = y {
            if let Some(o) = orientation {
                return Ok(Move::WallAt(o, Position {x, y}));
            } else {
                return Ok(Move::PawnTo(Position {x, y}));
            }
        }
    }

    Err(())
}


pub fn printer(mv :&Move) -> String {
    unimplemented!()
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn board() {
        assert_eq!(parse("e9").unwrap(), Move::PawnTo(Position { x: 5, y: 9 }));
        assert_eq!(parse("e9h").unwrap(), Move::WallAt(Orientation::Horizontal, Position { x: 5, y: 9 }));
        assert!(parse("e9hz").is_err());
    }

}
