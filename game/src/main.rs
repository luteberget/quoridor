use model::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::{thread,time};

static INDEX_HTML :&'static [u8] = include_bytes!("index.html");
static D3_JS :&'static [u8] = include_bytes!("d3/d3.js");

struct ServerThread { 
    current_board: Arc<Mutex<Board>>,
    player1_channel: Option<mpsc::Sender<Move>>,
    player2_channel: Option<mpsc::Sender<Move>>,
}

impl ws::Handler for ServerThread {
    fn on_message(&mut self, msg :ws::Message) -> ws::Result<()> { 
        eprintln!("MESSAGE from web client {:?}", msg);
        if let ws::Message::Text(txt) = msg {
            if let Ok(mv) = parse(&txt) {
                let current_board = self.current_board.lock().unwrap();
                if current_board.player == 0 {
                    if let Some(ch) = &self.player1_channel {
                        ch.send(mv).unwrap();
                    } else {
                        eprintln!("Not expecting this player to move.");
                    }
                }
                if current_board.player == 1 {
                    if let Some(ch) = &self.player2_channel {
                        ch.send(mv).unwrap();
                    } else {
                        eprintln!("Not expecting this player to move.");
                    }
                }
            } else {
                eprintln!("Could not parse move.");
            }
        } else {
            eprintln!("Received unexpected message type.");
        }

        Ok(()) 
    }
    fn on_request(&mut self, req :&ws::Request) -> ws::Result<ws::Response> {
        match req.resource() {
            "/ws" => ws::Response::from_request(req),
            "/" => Ok(ws::Response::new(200, "OK", INDEX_HTML.to_vec())),
            "/d3.js" => Ok(ws::Response::new(200, "OK", D3_JS.to_vec())),
            _ => Ok(ws::Response::new(404, "Not Found", b"404 - Not found".to_vec())),
        }
    }
}



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

    //let launch_ws = true;


    let current_board : Arc<Mutex<Board>> = Arc::new(Mutex::new(Default::default()));
    //let mut log_move :Box<FnMut(Move,&Board)> = Box::new(|_,_| {});

    let ws_player1 = true;
    let ws_player2 = true;

        let port = 9033;
        let addr = format!("localhost:{}", port);
        let current_board_ws = current_board.clone();

        let (p1_ch_tx, p1_ch_rx) = if ws_player1 { let (tx,rx) = mpsc::channel(); (Some(tx),(Some(rx))) } else{ (None,None) };
        let (p2_ch_tx, p2_ch_rx) = if ws_player2 { let (tx,rx) = mpsc::channel(); (Some(tx),(Some(rx))) } else{ (None,None) };

        let http = ws::WebSocket::new(move |out :ws::Sender| {
            eprintln!("New connection ({:?})", out.connection_id());
            let current_board = current_board_ws.lock().unwrap();
            let b = &*current_board;
            let send_move = (b.player == 0 && ws_player1) || (b.player == 1 && ws_player2);
            out.send(
                serde_json::to_string_pretty(&serde_json::json!({
                    "board": serde_json::to_value(b).unwrap(),
                    "send_move": send_move,
                })).unwrap()).unwrap();
            ServerThread {
                current_board: current_board_ws.clone(),
                player1_channel: p1_ch_tx.clone(),
                player2_channel: p2_ch_tx.clone(),
            }
        }).unwrap();

        let broadcaster = http.broadcaster();
        let current_board_log = current_board.clone();
        let log_move :Box<FnMut(Move,&Board)>= Box::new(move |m,b| {
            eprintln!("executed MOVE {:?} -- sending to ws", m);
            let mut current_board = current_board_log.lock().unwrap();
            *current_board = b.clone();
            let send_move = (b.player == 0 && ws_player1) || (b.player == 1 && ws_player2);
            broadcaster.send(
                serde_json::to_string_pretty(&serde_json::json!({
                    "board": serde_json::to_value(b).unwrap(),
                    "send_move": send_move,
                })).unwrap()).unwrap();
        });


        thread::spawn(move || {
            eprintln!("Info: Starting web server.");
            http.listen(addr).unwrap();
        });


        let mut p1 = WSPlayer {   rx: p1_ch_rx.unwrap() };
        let mut p2 = WSPlayer {   rx: p2_ch_rx.unwrap() };


    let addr = format!("http://{}/","localhost:9033");

    // Open web browser after starting server.
    thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(100));
        eprintln!("Info: Opening web browser, address {}", &addr);
        webbrowser::open(&addr).unwrap();
    });



    match play(&mut p1, &mut p2, log_move) {
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

use std::sync::mpsc;
pub struct WSPlayer {
    rx :mpsc::Receiver<Move>,
}

impl Player for WSPlayer {
    fn mv(&mut self, _mv :Option<Move>) -> Move {
        self.rx.recv().unwrap()
    }
    fn reset(&mut self) {}
}

/// Play two players against each other, returning the color
/// of the player that won.
fn play<A: Player, B: Player>(p1 :&mut A, p2: &mut B, mut log :Box<FnMut(Move,&Board)>) -> Result<usize,usize> {
    let mut board : Board = Default::default();

    // First move
    let mut last_move = p1.mv(None);
    loop {
        board.integrate(last_move).map_err(|_| 1usize)?; // Lose by foul
        log(last_move, &board);
        if let Some(winner) = board.get_winner() { return Ok(winner); }

        last_move = p2.mv(Some(last_move));
        board.integrate(last_move).map_err(|_| 0usize)?; // Lose by foul
        log(last_move, &board);
        if let Some(winner) = board.get_winner() { return Ok(winner); }

        last_move = p1.mv(Some(last_move));
    }
}

