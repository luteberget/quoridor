use model::*;

pub fn stdin_bot(mut player :impl Player) {
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let move_out = match line.unwrap().as_str() {
            "start" => player.mv(None),
            x => {
                let move_in = parse(x).unwrap();
                player.mv(Some(move_in))
            },
        };

        println!("{}", printer(&move_out));
    }
}
