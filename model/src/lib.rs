pub type Pos = (u8,u8); // (1-9, 1-9)
pub type Wall = (Pos,Pos);


#[derive(Clone,Copy)]
pub enum PlayerColor { Blue /* Player 1 */, Red /* Player 2 */ }
pub trait Player {
    /// A player receives a move (or None if the player should perform the first move)
    /// and must respond with a move.
    fn mv(&mut self, mv :Option<Move>) -> Move;
}

// When placing a horizontal wall:
//   param x in [1,8]
//   param y in [1,8]
//   mark below the left side of the wall as (x,y) 
//
// When placing a vertical wall:
//   ...
//   mark left of the top of the wall as (x,y)
//
// In total 2x u64
//

pub struct Board {
  pub horizontal_walls: u64,
  pub vertical_walls: u64,
  pub p1: Pos,
  pub p1_walls: u8,
  pub p2: Pos,
  pub p2_walls: u8,
  // whose TURN?
} // 64 + 64 + 16 + 16 bits

impl Default for Board {
    fn default() -> Board {
        Board {
            p1: (5,9), // e9
            p2: (5,1), // e1
            walls: vec![],
        }
    }
}

impl Board {
    pub fn integrate(self, mv :&Move) -> Result<Board, ()> {
        unimplemented!()
    }
}


// Should fit in 64 bits?
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Move {
    PawnTo(Pos),
    WallAt(Wall),
}


pub fn moves(board :&Board) -> Vec<Move> { // ask which player?
    unimplemented!()
}

/// Return number of the player that has won, or None if the game is not finished.
pub fn is_finished(board :&Board) -> Option<PlayerColor> {
    unimplemented!()
}

pub fn is_valid_move(board :&Board, mov :&Move) -> bool {
    for i in moves(board) {
        if i == *mov { return true; }
    }
    false
}


// The rules are:
// 1. move player into adjacent free space
// 2. insert wall 
//    - that does not block any player's path
// 3. jump over another player, 
//    - or if there is a wall, jump diagonally over them.
