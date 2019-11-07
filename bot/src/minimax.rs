use std::collections::{HashMap, VecDeque};
use arrayvec::ArrayVec;
use model::*;
use log::*;


pub struct HeuristicBot {
    board :Board,
}

impl HeuristicBot {
    pub fn new(board :Board) -> HeuristicBot {
        HeuristicBot { board }
    }
}

impl Player for HeuristicBot {
    fn reset (&mut self) {}

    fn mv(&mut self, mv :Option<Move>) -> Move {
        if let Some(mv) = mv { self.board.integrate(mv).unwrap(); }
        let i_am_player = self.board.player;
        info!("HeuristicPlayer Received move {:?} as player {}", mv, i_am_player);
        let mut  moves :Vec<(Move,f32)>= Vec::new();
        //let (mut score, mut mv) = (-std::f32::INFINITY, None);
        for_each_move(&self.board, &mut |m| {
            let mut new_board = self.board.clone();
            new_board.integrate(m).unwrap();
            //debug!("Evaluating heuristic for {:?}", new_board);
            let new_score = (1.0-2.0*(i_am_player as f32))*board_heuristic(&new_board); // board heuristic always
            //debug!("  Score: {}", new_score);
            //if new_score >= score {
            //    score = new_score;
            //    mv = Some(m);
            //}

            moves.push((m,new_score));

            true
        });

        moves.sort_by_key(|&(a,b)| ordered_float::OrderedFloat(-b));
        for (mv,sc) in moves.iter().take(5) {
            debug!("move {:?} score {}", mv, sc);
        }
        let mv = moves.get(0).map(|x| x.0);

        self.board.integrate(mv.unwrap()).unwrap();
        mv.unwrap()

    }
}

pub struct MinimaxPlayer {
    board :Board,
    memory :HashMap<Board, BoardInfo>,
}

impl MinimaxPlayer {
    pub fn new() -> MinimaxPlayer {
        MinimaxPlayer {
            board: Default::default(),
            memory: Default::default(), // TODO could have precomputed moves here?
        }
    }
}

impl Player for MinimaxPlayer {

    fn reset(&mut self) {}  // Keep the table until next game.

    fn mv(&mut self, mv :Option<Move>) -> Move {
        let depth = 5;
        if let Some(mv) = mv { self.board.integrate(mv).unwrap(); }
        let best_move = negamax_root(&mut self.memory, &self.board, depth);
        self.board.integrate(best_move).unwrap();
        best_move
    }
}

pub fn for_each_move(board :&Board, f :&mut dyn FnMut(Move) -> bool) {
    if !for_each_pawn_move(board, f) { return; }
    if !for_each_wall_move(board, f) { return; }
}

/// available pawn moves
pub fn for_each_pawn_move(board :&Board,  f :&mut dyn FnMut(Move)->bool) -> bool {
    let current_pos = board.positions[board.player];
    let other_pos   = board.positions[1 - board.player];

    let candidates = 
        vec! [ 
        Position { x: current_pos.x + 1, y: current_pos.y },
        Position { x: current_pos.x - 1, y: current_pos.y },
        Position { x: current_pos.x , y: current_pos.y + 1},
        Position { x: current_pos.x , y: current_pos.y - 1},
        ];

    for candidate in candidates {
        if in_bounds1to9(&candidate) && !board.wall_between(&current_pos, &candidate) {
            if candidate != other_pos {
                let cont = f(Move::PawnTo(candidate));
                if !cont { return false; }
            } else {
                let other_side = Position { x: other_pos.x + (other_pos.x - current_pos.x),
                                            y: other_pos.y + (other_pos.y - current_pos.y)};
                if board.wall_between(&&candidate, &other_side) {
                    // There is a back wall after the other player, diagonal jumps allowed

                    for sign in vec![-1,1] {
                        let diag = Position { x: other_pos.x + sign*(other_pos.y - current_pos.y),
                                               y: other_pos.y + sign*(other_pos.x - current_pos.x)};
                        if in_bounds1to9(&diag) && !board.wall_between(&diag, &other_pos) {
                            let cont = f(Move::PawnTo(diag));
                            if !cont { return false; }
                        }
                    }
                }
            }
        }
    }

    return true; // continue outer loop 
}

/// Available wall moves
pub fn for_each_wall_move(board :&Board, f : &mut dyn FnMut(Move)->bool) -> bool{
    for orientation in vec![Orientation::Horizontal, Orientation::Vertical] {
        for x in 0..=8 {
            for y in 0..=8 {
                if board.can_add_wall(orientation, Position { x, y }) {
                    let cont = f(Move::WallAt(orientation, Position { x,  y }));
                    if !cont { return false; }
                }
            }
        }
    }

    return true; // continue outer loop
}

pub fn for_each_nonwalled_neighbor(board :&Board, cell :Position, mut f :impl FnMut(Position)) {
    let candidates = 
        vec! [ 
        Position { x: cell.x + 1, y: cell.y },
        Position { x: cell.x - 1, y: cell.y },
        Position { x: cell.x , y: cell.y + 1},
        Position { x: cell.x , y: cell.y - 1},
        ];

    for candidate in candidates {
        if in_bounds1to9(&candidate) && !board.wall_between(&cell, &candidate) {
            f(candidate);
        }
    }
}

#[test]
fn test_effective_resistance() {
    println!("effective resistance test.");

    let player = 0;
    let mut board :Board = Default::default();
    board.positions[player].y = 1;
    board.positions[player].x = 5;

    board.walls.push((Orientation::Horizontal, Position { x: 1, y: 1 }));
    board.walls.push((Orientation::Horizontal, Position { x: 2, y: 1 }));
    board.walls.push((Orientation::Horizontal, Position { x: 3, y: 1 }));
    board.walls.push((Orientation::Horizontal, Position { x: 4, y: 1 }));
    board.walls.push((Orientation::Horizontal, Position { x: 5, y: 1 }));
    board.walls.push((Orientation::Horizontal, Position { x: 6, y: 1 }));

    let x = effective_resistances(&board, player);

    print_board_resistance(x.as_slice(), player);

    println!(" in total {}", effective_resistance(&board, player));
}

pub fn print_board_resistance(values :&[f64], player :usize) {
    let goal_y = if player == 0 { 9 } else { 1 };
    let encode_lsqr = |cell :Position| 
        if cell.y == goal_y { 0 } 
        else if cell.y > goal_y { 1+ encode9(cell.x,cell.y-1) }
        else { 1+ encode9(cell.x,cell.y) };

    let mut tbl = prettytable::Table::new();
    for y in (1..=9).rev() {
        let mut row = prettytable::Row::empty();
        for x in (1..=9) {
            let resistance = values[encode_lsqr(Position {x, y })];
            row.add_cell(prettytable::Cell::new(&format!("{:.2}",resistance)));
        }
        tbl.add_row(row);
    }
    tbl.printstd();
}

pub fn effective_resistance(board :&Board, player :usize) -> f64 {
    let goal_y = if player == 0 { 9 } else { 1 };
    let encode_lsqr = |cell :Position| 
        if cell.y == goal_y { 0 } 
        else if cell.y > goal_y { 1+ encode9(cell.x,cell.y-1) }
        else { 1+ encode9(cell.x,cell.y) };

    let resistances = effective_resistances(board, player);
    resistances[encode_lsqr(board.positions[player])]
}

pub fn effective_resistances(board :&Board, player :usize) -> Vec<f64> {

    let goal_y = if player == 0 { 9 } else { 1 };

    // Define a set of linear equations with one
    // variable for each cell on the grid. 
    // (unconnected cells should be fine with least squares method)
    //
    // Goal row is a single node with net current -1
    // Player cell is a node with net current 1
    // The rest of the cells have net current 0 and connections to 
    // adjacent cells when there is no wall between.
    // Goal row has potential 0.
    // Check the player cell node's potential for the effective resistance 
    // of the board.
    //
    // Assumption: player is not in their goal row.


    // number of variables: all cells except for the goal row, which is one supernode
    let n_cols = 9*8 +1; 

    // number of equations: one per node, plus setting goal row potential to zero.
    let n_rows = (9*8+1) + 1;

    let encode_lsqr = |cell :Position| 
        if cell.y == goal_y { 0 } 
        else if cell.y > goal_y { 1+ encode9(cell.x,cell.y-1) }
        else { 1+ encode9(cell.x,cell.y) };

    let mut rhs = vec![0.0;n_rows];
    rhs[0] = -1.0;
    rhs[encode_lsqr(board.positions[player])] = 1.0;

    let aprod = |mode : lsqr::Product| {
        match mode {
            lsqr::Product::YAddAx { x , y } => {
                // y += A*x   [m*1] = [m*n]*[n*1]

                // Each link between cells adds a resistor
                // between their nodes,
                //
                for row in (1..=9).filter(|r| *r != goal_y) {
                    for col in 1..=9 {
                        let pos = Position { x: col, y: row };
                        for_each_nonwalled_neighbor(board, pos, |other|  {
                            //println!("pos {:?}", pos);
                            let idx_a = encode_lsqr(pos);
                            let idx_b = encode_lsqr(other);
                            y[idx_a] += x[idx_a] - x[idx_b];
                        });
                    }
                }

                // Also, goal row has potential zero.
                // This is the bottom row of the matrix, containing only the first variable
                // (the goal row supernode)
                y[n_rows-1] += x[0];

            },
            lsqr::Product::XAddATy { x, y } => {
                // x += A^T*y  [n*1] = [n*m][m*1]

                for row in (1..=9).filter(|r| *r != goal_y) {
                    for col in 1..=9 {
                        let pos = Position { x: col, y: row };
                        for_each_nonwalled_neighbor(board, pos, |other| {
                            let idx_a = encode_lsqr(pos);
                            let idx_b = encode_lsqr(other);
                            x[idx_a] += y[idx_a];
                            x[idx_b] -= y[idx_a];
                        });
                    }
                }

                // the goal row has potential zero
                x[0] += y[n_rows-1]; // ???? 
            },
        }
    };

    let params = lsqr::Params {
        condlim: 0.0, damp: 0.0, iterlim: 10000, rel_mat_err: 0.0, rel_rhs_err: 0.0
    };

    let (sol,statistics)  = lsqr::lsqr(|msg| println!("{}", msg),
        n_rows, n_cols, params, aprod, &mut rhs);

    sol
}

/// measure the board's worth directly
pub fn board_heuristic(board :&Board) -> f32 {
    // assumptions:
    // positive number is player1 advantage
    // zero-sum =>  score(player1,board) = -score(player2,board)
    // 
    // ideas:
    //  1. shortest path to goal
    //
    //  2. number or "wideness" of path somehow.
    //     maybe there is a short path to the goal which can easily
    //     be blocked by the other player. So some sort of robustness
    //     to walling seems to be better than just the shortest path.
    //
    //     = max flow / min cut / min horizontal cut / shortest non-path from 
    //       left of the board to right of the board
    //     BUT it might not matter if you have gotten past the bottle-neck.
    //
    //  3. number of walls you have left
    //
    //  4. jumping should probably be ignored?
    //  
    //  5. if the number of opponent's walls left is not enough
    //     to block your path, give a very high score on the difference
    //     between shortest path for each player.
    //
    //  6. irrelevant wall moves could maybe be eliminated by a heuristic.
    //     because there are some obvious bad choices, for example setting
    //     a wall at a corner at the beginning of the game. Maybe
    //     these can be eliminated from all consideration by other methods.
    //
    // parameters:
    //  1. balancing between walls left and good paths.
    //  2. the minimum and maximum "flow" number that is possible 
    //     from the player as source to the goal row as sink.
    //
    //
    // Each of these can be measured for each player, taking the
    // difference score(p1) - score(p2).
    //
    
    if let Some(winner) = board.get_winner() {
        let sign = -( (winner as f32) * 2.0 - 1.0);
        return (sign as f32) *std::f32::INFINITY;
    }

    let wall_weight = 0.05;

    //
    // (f_a - f_b) + p*(w_a - w_b)
    //

    let r0 = effective_resistance(&board, 0);
    let r1 = effective_resistance(&board, 1);

    let f = (r1 - r0) as f32;

    (f + wall_weight*(board.walls_left[0] as f32 - board.walls_left[1] as f32 )) as f32
}

pub fn for_each_adjacent_cell(board :&Board, pos :Position,mut f:impl FnMut(Position)) {
    let candidates = [ 
        Position { x: pos.x + 1, y: pos.y },
        Position { x: pos.x - 1, y: pos.y },
        Position { x: pos.x ,    y: pos.y + 1},
        Position { x: pos.x ,    y: pos.y - 1},
        ];

    for c in candidates.iter() {
        if in_bounds1to9(c) && !board.wall_between(&pos, c) {
            f(*c);
        }
    }
}

pub fn player_flow(board :&Board, player :usize) -> u64 {
    // USE only encode9 positions inside this function with type isize
    //
    let pos = board.positions[player];
    debug!("player_flow, pos= {:?}", pos);
    let pos = encode9(pos.x, pos.y) as isize;

    let mut edges : Vec<(u64,ArrayVec<[isize; 4]>)> = vec![(0,ArrayVec::new()); 9*9];
    let mut queue : VecDeque<isize> = Default::default();

    queue.push_back(pos);
    edges[pos as usize] = (100, ArrayVec::new());

    while let Some(p) = queue.pop_front() {
        for_each_adjacent_cell(board, decode9(p as usize), |q| {
            //trace!("Adjacent cell {:?}", q);
            let q = encode9(q.x,q.y) as isize;
            // add a link from p to q
            edges[p as usize].1.push(q);

            // did we reach a new node?
            if edges[q as usize].0 == 0 {
                // then add it with a new capacity
                let capacity = edges[p as usize].0;
                let new_capacity = (capacity - 1).max(1);
                edges[q as usize].0 = new_capacity;
                queue.push_back(q);
            }
        });
    }

    fn find_path(source :isize,
                 residual :&Vec<(u64, ArrayVec<[isize;4]>)>, 
                 parent :&mut [isize;81], 
                 end :&mut isize,
                 goal_y :usize) -> bool{

        //if residual[source as usize].0 <= 0 { return false; }

        let mut visited = [false;81];
        let mut queue = VecDeque::new();
        queue.push_back(source);
        visited[source as usize] = true;
        while let Some(p) = queue.pop_front() {

            if decode9(p as usize).y == goal_y as i64 { // decode9(p).y == goal_y
                *end = p;
                return true;
            }

            for q in &residual[p as usize].1 {
                if residual[*q as usize].0 > 0 {
                    //trace!("Residual OK {:?} {:?} --> {:?} {:?} {}", 
                    //       p, decode9(p as usize), 
                    //       q, decode9(*q as usize), 
                    //       residual[p as usize].0);
                    if !visited[*q as usize] {
                        queue.push_back(*q);
                        parent[*q as usize] = p;
                        visited[*q as usize] = true;
                    }
                }
            }
        }

        return false;
    }

    // now we have a graph reprsenting the "flow" of score   so that
    // a shorter and wider path from the player to the goal gives a higher score.

    // use Edmonds-Karp to find this maximum flow
    let mut parent = [-1isize;81];
    let mut end = -1isize;
    let mut flow = 0u64;
    let mut residual = edges;
    //let source = encode9(player.x, player.y) as isize;
    let source = pos as isize;
    debug!("*** MAX FLOW");
    //debug!("Residual flow {:?}", residual);
    //debug!("Starting from source {:?} {:?}", source, decode9(source as usize));
    let goal_y = if player == 0 { 9 } else { 1 };
    while find_path(source, &residual, &mut parent, &mut end, goal_y) {
        //trace!("Found path to {:?} {:?}", end, decode9(end as usize));
        let mut path_flow = 100;
        let mut n = end;
        while n != source {
            path_flow = path_flow.min(residual[n as usize].0);
            //trace!("Through {:?} {:?}, flow max {:?}", n, decode9(n as usize), path_flow);
            n = parent[n as usize];
            //trace!("  - came from {:?} {:?}", n, decode9(n as usize));
        }
        flow += path_flow;
        let mut n = end;
        while n != source {
            residual[n as usize].0 -= path_flow;
            n = parent[n as usize];
        }
    }
    debug!("   - {}", flow);

    flow
}

#[repr(u16)]
pub enum BoardFlag { Exact, LowerBound, UpperBound }
pub struct BoardInfo {
    pub value: f32,
    pub depth: u16,
    pub flag :BoardFlag,
} // size should be 64bit

pub fn negamax_root(table :&mut HashMap<Board, BoardInfo>, board :&Board,
                    depth: u16) -> Move {

    let (mut score, mut mv) = (-std::f32::INFINITY, None);
    for_each_move(&board, &mut |m| {
        let mut new_board = board.clone();
        new_board.integrate(m).unwrap();
        let new_score = -negamax(table, &new_board, depth, -std::f32::INFINITY, std::f32::INFINITY);
        if new_score >= score {
            score = new_score;
            mv = Some(m);
        }
        true
    });

    mv.unwrap()
}

pub fn negamax(table :&mut HashMap<Board, BoardInfo>, board :&Board,
               depth: u16, mut alpha :f32, mut beta :f32) -> f32 {
    let alpha_original = alpha;

    // check the table
    if let Some(info) = table.get(board) {
        if info.depth >= depth {
            match info.flag {
                BoardFlag::Exact => { return info.value; },
                BoardFlag::LowerBound => { alpha = alpha.max(info.value); },
                BoardFlag::UpperBound => { beta = beta.min(info.value); },
            };

            if alpha >= beta { return info.value; }
        }
    }

    if depth == 0 || board.get_winner().is_some() {
        return ((1-2*board.player) as f32)*board_heuristic(board); // board heuristic always
        // takes the perspective of player 1 (first), so we multiply by the current player we
        // are looking at.
    }

    let mut value = - std::f32::INFINITY;
    // TODO order moves by heuristic?
    for_each_move(&board, &mut |m| {
        let mut new_board = board.clone();
        new_board.integrate(m).unwrap(); // panic if we generated an invalid move
        let new_value = -negamax(table, &new_board, depth -1, -beta, -alpha);
        value = value.max(new_value);
        alpha = alpha.max(value);
        if alpha >= beta {
            false // break for_each_move
        } else {
            true // continue for_each_move
        }
    });


    // TODO: merge boards with similar numbre of remaining walls
    //  --- you might not play very differently if you have 9 remaining
    //  walls that if you have 10. So it might be good to use the information
    //  about one to guide the other.

    let new_info = BoardInfo {
        value: value,
        depth: depth,
        flag: if value <= alpha_original {
            BoardFlag::UpperBound
        } else if value >= beta {
            BoardFlag::LowerBound
        } else {
            BoardFlag::Exact
        }
    };

    table.insert(board.clone(), new_info);
    value
}


