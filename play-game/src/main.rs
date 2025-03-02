mod game;
mod tests;
mod transaction;
mod util;

use util::{get_payer_key, get_program_id};

fn main() {
    let program_id = get_program_id();
    let payer = get_payer_key();
    tests::play_player_one_wins_game(&program_id, &payer);
    tests::tie_game(&program_id, &payer);
}
