#![allow(unexpected_cfgs)]

mod game;
mod transaction;
mod util;

use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use util::{get_payer_key, get_program_id, get_rpc_client};

use crate::game::{
    Game, GameAccount,
    GameState::*,
    Sign::{O, X},
};

fn new_game(program_id: &Pubkey, payer: &Keypair) -> Game {
    let player_one = Keypair::new();
    let player_two = Keypair::new();

    let rpc_client = get_rpc_client(CommitmentConfig::finalized());

    let lamports = native_token::sol_to_lamports(1.0);
    transaction::transfer(&rpc_client, lamports, payer, &player_one.pubkey());
    transaction::transfer(&rpc_client, lamports, payer, &player_two.pubkey());

    let game = Game::new(*program_id, rpc_client.url(), player_one, player_two);
    game.setup_game();
    game
}

fn play_player_one_wins_game(program_id: &Pubkey, payer: &Keypair) {
    let mut game = new_game(program_id, payer);

    assert_eq!(
        game.play((0, 0)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), None, None],
                [None, None, None],
                [None, None, None]
            ],
            turn: 2,
        }
    );

    assert_eq!(
        game.play((1, 0)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), None, None],
                [Some(O), None, None],
                [None, None, None]
            ],
            turn: 3,
        }
    );

    assert_eq!(
        game.play((0, 1)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), Some(X), None],
                [Some(O), None, None],
                [None, None, None]
            ],
            turn: 4,
        }
    );

    assert_eq!(
        game.play((1, 1)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), Some(X), None],
                [Some(O), Some(O), None],
                [None, None, None]
            ],
            turn: 5,
        }
    );

    assert_eq!(
        game.play((0, 2)),
        GameAccount {
            players: game.players(),
            state: Won {
                winner: game.player_one.pubkey()
            },
            board: [
                [Some(X), Some(X), Some(X)],
                [Some(O), Some(O), None],
                [None, None, None]
            ],
            turn: 5, // turn doesn't increment after the game ends
        },
    );
}

fn main() {
    let program_id = get_program_id();
    let payer = get_payer_key();
    play_player_one_wins_game(&program_id, &payer);
}
