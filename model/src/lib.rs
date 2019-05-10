pub type Pos = (u8,u8); // (1-9, 1-9)
pub type Wall = (Pos,Pos);

pub struct Board {
  p1: Pos,
  p2: Pos,
  walls: Vec<Wall>,
}


// Should fit in 64 bits
#[derive(PartialEq, Eq)]
pub enum Move {
    Wall(Wall),
    Player1(Pos),
    Player2(Pos),
}


pub fn moves(board :&Board) -> Vec<Move> { // ask which player?
    unimplemented!()
}

/// Return number of the player that has won, or None if the game is not finished.
pub fn is_finished(board :&Board) -> Option<u8> {
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
