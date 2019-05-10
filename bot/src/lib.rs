use model::*;

/// measure the board's worth directly
pub fn board_heuristic(board :&Board) -> f64 {
}


/// Single step minimax solver
pub fn minimax_step(board :&Board) -> Move {

}


/// Single step of a minimax solver, but keep caches around.
pub fn minimax_cached(solver :&mut SolverState, this_player :bool, 
                      depth :usize, board :&Board) -> f64 {

    // Base case
    if depth == 0 || is_finished(board) {
        return board_heuristic(board);
    }

    // This player
    if this_player {
        let mut value = -inf;
        for mov in moves(board) {
            let new_board = board.apply(mov);
            let new_board_value = minimax_cached(solver, false, depth-1, new_board);
            value = value.max(new_board_value);
        }
        value
    } else {
        // Other player
        let mut value = inf;
        for mov in moves(board) {
            let new_board = board.apply(mov);
            let new_board_value = minimax_cached(solver, true, depth-1, new_board);
            value = value.min(new_board_value);
        }
        value
    }
}

// alpha beta pruning TODO
//
//
//    function alphabeta(node, depth, α, β, maximizingPlayer) is
//        if depth = 0 or node is a terminal node then
//            return the heuristic value of node
//        if maximizingPlayer then
//            value := −∞
//            for each child of node do
//                value := max(value, alphabeta(child, depth − 1, α, β, FALSE))
//                α := max(α, value)
//                if α ≥ β then
//                    break (* β cut-off *)
//            return value
//        else
//            value := +∞
//            for each child of node do
//                value := min(value, alphabeta(child, depth − 1, α, β, TRUE))
//                β := min(β, value)
//                if α ≥ β then
//                    break (* α cut-off *)
//            return value
//    (* Initial call *)
//    alphabeta(origin, depth, −∞, +∞, TRUE)
