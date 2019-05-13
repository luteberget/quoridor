use model::*;

static INDEX_HTML :&'static [u8] = include_bytes!("index.html");
static D3_JS :&'static [u8] = include_bytes!("d3/d3.js");

struct ServerThread { }

impl ws::Handler for ServerThread {
    fn on_message(&mut self, _msg :ws::Message) -> ws::Result<()> { Ok(()) }
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

    let launch_ws = true;

    use std::sync::Arc;
    use std::sync::Mutex;
    use std::{thread,time};

    let current_board : Arc<Mutex<Board>> = Arc::new(Mutex::new(Default::default()));
    let mut log_move :Box<FnMut(Move,&Board)> = Box::new(|_,_| {});

    if launch_ws {
        let port = 9033;
        let addr = format!("localhost:{}", port);
        let current_board_ws = current_board.clone();
        let http = ws::WebSocket::new(move |out :ws::Sender| {
            eprintln!("New connection ({:?})", out.connection_id());
            let current_board = current_board_ws.lock().unwrap();
            out.send(serde_json::to_string_pretty(&*current_board).unwrap()).unwrap();
            ServerThread {}
        }).unwrap();

        let broadcaster = http.broadcaster();
        let current_board_log = current_board.clone();
        log_move = Box::new(move |m,b| {
            eprintln!("executed MOVE {:?} -- sending to ws", m);
            let mut current_board = current_board_log.lock().unwrap();
            *current_board = b.clone();
            broadcaster.send(serde_json::to_string_pretty(b).unwrap()).unwrap();
        });


        thread::spawn(move || {
            eprintln!("Info: Starting web server.");
            http.listen(addr).unwrap();
        });
    }


    let addr = format!("http://{}/","localhost:9033");

    // Open web browser after starting server.
    thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(100));
        eprintln!("Info: Opening web browser, address {}", &addr);
        webbrowser::open(&addr).unwrap();
    });



    let mut p1 = CLIPlayer { name: "p1"};
    let mut p2 = CLIPlayer { name: "p2"};
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

