use model::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::{thread,time};
use std::io::{BufRead, Write};

pub struct ChannelPlayer {
    pub tx :mpsc::Sender<Option<Move>>,
    pub rx :mpsc::Receiver<Move>,
}

impl Player for ChannelPlayer {
    fn mv(&mut self, mv:Option<Move>) -> Move {
        self.tx.send(mv).unwrap();
        self.rx.recv().unwrap()
    }

    fn reset(&mut self) {}
}

impl ChannelPlayer {
    pub fn from_thread<F2: FnOnce() + Send + 'static>
        (f :impl FnOnce(mpsc::Receiver<Option<Move>>, mpsc::Sender<Move>) -> F2) -> Self {
        let (input_tx,input_rx) = mpsc::channel();
        let (output_tx,output_rx) = mpsc::channel();
        thread::spawn(f(input_rx, output_tx));
        ChannelPlayer {
            tx: input_tx,
            rx: output_rx,
        }
    }
}

pub fn protocol(mut r :impl BufRead, mut w: impl Write, 
                input :&mpsc::Receiver<Option<Move>>, output :&mpsc::Sender<Move>) {
        loop {
            // Send move
            let mv = input.recv().unwrap();
            match mv {
                None => w.write("start\n".as_bytes()).unwrap(),
                Some(mv) => w.write(format!("{}\n",printer(&mv)).as_bytes()).unwrap(),
            };

            // Receive answer
            let mut line :String = String::new();
            let mv_str = r.read_line(&mut line).unwrap();
            let mv = parse(line.lines().next().unwrap()).unwrap();
            output.send(mv).unwrap();
        }
}

pub fn stdio_player(program :String) -> impl Player {
    ChannelPlayer::from_thread(|input,output| move || {
        use std::process::*;
        use std::io::Write;

        let args = shell_words::split(&program).unwrap();
        eprintln!("Launching process {:?}", args);
        let proc = Command::new(&args[0]).args(&args[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn().unwrap();
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(proc.stdout.unwrap());
        protocol(reader, proc.stdin.unwrap(), &input, &output);
    })
}

pub fn net_player(port :u32) -> impl Player {
    ChannelPlayer::from_thread(|input,output| move || {
        use std::thread;
        use std::net::{TcpListener, TcpStream, Shutdown};
        use std::io::{Read, Write};

        let addr = format!("0.0.0.0:{}",port);
        let listener = TcpListener::bind(&addr).unwrap();
        eprintln!("Waiting for connection on {}", addr);
        match listener.accept() {
            Ok((stream,addr)) => {
                eprintln!("New connection: {}", addr);
                use std::io::BufRead;
                let mut reader = std::io::BufReader::new(&stream);
                protocol(reader, &stream, &input, &output);
            },
            Err(_) => {
            },
        };
    })
}


static INDEX_HTML :&'static [u8] = include_bytes!("index.html");
static D3_JS :&'static [u8] = include_bytes!("d3/d3.js");

struct ServerThread { 
    current_board: Arc<Mutex<Board>>,
    out: ws::Sender,
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
                        if current_board.is_valid_move(&mv) {
                            ch.send(mv).unwrap();
                        } else {
                            self.out.send(serde_json::to_string_pretty(&serde_json::json!({
                                "message": "Invalid move",
                            })).unwrap()).unwrap();
                        }
                    } else {
                        eprintln!("Not expecting this player to move.");
                    }
                }
                if current_board.player == 1 {
                    if let Some(ch) = &self.player2_channel {
                        if current_board.is_valid_move(&mv) {
                            ch.send(mv).unwrap();
                        } else {
                            self.out.send(serde_json::to_string_pretty(&serde_json::json!({
                                "message": "Invalid move",
                            })).unwrap()).unwrap();
                        }
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

struct Opts {
    show_gui: bool,
    verbose: bool,
    p1: Box<Player>,
    p2: Box<Player>,
}

fn get_opts() -> Result<Opts,&'static str> {
    use std::env;

    let mut show_gui = false;
    let mut verbose = false;
    let mut p1 :Option<Box<Player>> = None;
    let mut p2 :Option<Box<Player>> = None;
    let mut web_players = (None,None);

    let mut args = env::args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-g" => { show_gui = true; },
            "-v" => { verbose = true; },
            "cli" => {
                if p1.is_none() { p1 = Some(Box::new(CLIPlayer {name: "Player1" })); }
                else if p2.is_none() { p1 = Some(Box::new(CLIPlayer {name: "Player2"})); }
                else { return Err("More than two players requested."); }
            },
            "gui" => {
                if p1.is_none() { 
                    let (tx,rx) = mpsc::channel();
                    web_players.0 = Some(tx);
                    p1 = Some(Box::new(WSPlayer {rx}));

                }
                else if p2.is_none() { 
                    let (tx,rx) = mpsc::channel();
                    web_players.1 = Some(tx);
                    p2 = Some(Box::new(WSPlayer {rx}));
                }
                else { return Err("More than two players requested."); }
            },
            "run" => {
                let program : String = args.next().ok_or("Run program requires argument")?;
                if p1.is_none() { p1 = Some(Box::new(stdio_player(program))); }
                else if p2.is_none() { p1 = Some(Box::new(stdio_player(program))); }
                else { return Err("More than two players requested."); }
            },
            "net" => {
                let port :u32 = args.next().ok_or("Net program requires port")?
                    .parse::<u32>().map_err(|_| "Could not parse port number for net player.")?;
                if p1.is_none() { p1 = Some(Box::new(net_player(port))); }
                else if p2.is_none() { p1 = Some(Box::new(net_player(port))); }
                else { return Err("More than two players requested."); }
            },
            _ => { return Err("Unrecognized argument"); },
        }
    }
    

    Ok(Opts {
        show_gui: show_gui,
        verbose: verbose,
        p1: p1.unwrap(),
        p2: p2.unwrap(),
    })
}

fn main() {
    use std::env;
    eprintln!("Quoridor");
    
    struct Opts {
        show_gui :bool, //verbose: bool,
        p1: Box<Player>,
        p2: Box<Player>,
    }

    let mut opts = Opts { show_gui: false, 
        //verbose: bool,
        p1: Box::new(CLIPlayer{name: "p1"}),
        p2: Box::new(CLIPlayer{name: "p2"}) };

    let mut args = env::args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-g" => { opts.show_gui = true; },
            _ => panic!("arg err"),
        }
    }

    let current_board : Arc<Mutex<Board>> = Arc::new(Mutex::new(Default::default()));
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
                out: out,
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


fn play_dyn<'a> (p1 :&'a mut dyn Player, 
                 p2 :&'a mut dyn Player, 
                 log :&mut dyn FnMut(Move, &Board)) -> Result <usize,usize> {
    let mut board :Board = Default::default();
    let mut last_move :Option<Move> = None;
    let (mut current_player,mut next_player) = ((p1,0usize),(p2,1usize)); // Player 1 starts.
    loop {
        last_move = Some(current_player.0.mv(last_move));
        board.integrate(last_move.unwrap()).map_err(|_| next_player.1)?;
        log(last_move.unwrap(), &board);
        if let Some(winner) = board.get_winner() { return Ok(winner); }
        std::mem::swap(&mut current_player, &mut next_player);
    }
}


