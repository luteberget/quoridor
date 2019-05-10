use model::*;

// the 'game' exe starts
// - two players
// - and optionally a websocket server which sends new moves to visualization

fn main() {
    eprintln!("Quoridor");

    let mut p1 = CLIPlayer { name: "p1"};
    let mut p2 = CLIPlayer { name: "p2"};
    match play(&mut p1, &mut p2) {
        Ok(PlayerColor::Blue) => eprintln!("Blue player won!"),
        Ok(PlayerColor::Red) => eprintln!("Red player won!"),
        Err(PlayerColor::Blue) => eprintln!("Blue player won by move error!"),
        Err(PlayerColor::Red) => eprintln!("Red player won by move error!"),
    }
}

pub struct CLIPlayer { name :&'static str }
impl Player for CLIPlayer {
    fn mv(&mut self, mv :Option<Move>) -> Move {
        eprintln!("{}: received {:?}", self.name, mv);
        Move::PawnTo((0,0))
    }
}

/// Play two players against each other, returing the color
/// of the player that won.
fn play<A: Player, B: Player>(p1 :&mut A, p2: &mut B) -> Result<PlayerColor,PlayerColor> {
    let mut board : Board = Default::default();

    // First move
    let mut last_move = p1.mv(None);
    loop {
        board = board.integrate(&last_move).map_err(|_| PlayerColor::Red)?; // Lose by foul
        if board.p1.1 == 1 { return Ok(PlayerColor::Blue); } // Win by reaching end

        last_move = p2.mv(Some(last_move));
        board = board.integrate(&last_move).map_err(|_| PlayerColor::Blue)?; // Lose by foul
        if board.p2.1 == 9 { return Ok(PlayerColor::Red); } // Win by reaching end

        last_move = p1.mv(Some(last_move));
    }
}

