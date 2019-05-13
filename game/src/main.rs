use model::*;

// the 'game' exe starts
// - two players
// - and optionally a websocket server which sends new moves to visualization


// pub trait PipedTimeoutPlayer {
//     pub fn new() -> {
//         // start a process
//     }
// 
// }
// 
// impl Player for PipedTimeoutPlayer {
//     fn mv(&mut self, mv: Option<Move>) -> Move {
//         // Send to stdout
//         // Wait for response with timeout
//         // ...
//         unimplemented!()
//     }
// 
//     fn reset(&mut self) {}
// }


fn main() {
    eprintln!("Quoridor");

    let mut p1 = CLIPlayer { name: "p1"};
    let mut p2 = CLIPlayer { name: "p2"};
    match play(&mut p1, &mut p2) {
        Ok(0) => eprintln!("Blue player won!"),
        Ok(1) => eprintln!("Red player won!"),
        Err(0) => eprintln!("Blue player won by move error!"),
        Err(1) => eprintln!("Red player won by move error!"),
        _ => panic!(),
    }
}

pub struct CLIPlayer { name :&'static str }
impl Player for CLIPlayer {
    fn mv(&mut self, mv :Option<Move>) -> Move {
        eprintln!("{}: received {:?}", self.name, mv);
        use std::io::{self, BufRead};
        let line1 = io::stdin().lock().lines().next().unwrap().unwrap();
        parse(&line1).unwrap()
    }

    fn reset(&mut self) {}
}

/// Play two players against each other, returning the color
/// of the player that won.
fn play<A: Player, B: Player>(p1 :&mut A, p2: &mut B) -> Result<usize,usize> {
    let mut board : Board = Default::default();

    // First move
    let mut last_move = p1.mv(None);
    loop {
        board.integrate(last_move).map_err(|_| 1usize)?; // Lose by foul
        if let Some(winner) = board.get_winner() { return Ok(winner); }

        last_move = p2.mv(Some(last_move));
        board.integrate(last_move).map_err(|_| 0usize)?; // Lose by foul
        if let Some(winner) = board.get_winner() { return Ok(winner); }

        last_move = p1.mv(Some(last_move));
    }
}

