mod minimax;
mod bot;

use model::*;

fn main() {
    use env_logger::Env;
    env_logger::from_env(Env::default().default_filter_or("trace")).init();

    bot::stdin_bot(minimax::HeuristicBot::new(Default::default()));
}
