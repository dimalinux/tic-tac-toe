use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::game::{
    Game, GameAccount,
    GameState::{Active, Tie, Won},
    Sign::{O, X},
};

fn new_game<'a>(
    program_id: &'a Pubkey,
    rpc_client: &'a RpcClient,
    player_one: &'a Keypair,
    player_two: &'a Keypair,
) -> Game<'a> {
    let game = Game::new(program_id, rpc_client, player_one, player_two);
    game.setup_game();
    game
}

pub(crate) fn play_player_one_wins_game(
    program_id: &Pubkey,
    rpc_client: &RpcClient,
    player_one: &Keypair,
    player_two: &Keypair,
) {
    let mut game = new_game(program_id, rpc_client, player_one, player_two);

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

pub(crate) fn tie_game(
    program_id: &Pubkey,
    rpc_client: &RpcClient,
    player_one: &Keypair,
    player_two: &Keypair,
) {
    let mut game = new_game(program_id, rpc_client, player_one, player_two);

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
        game.play((1, 1)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), None, None],
                [None, Some(O), None],
                [None, None, None]
            ],
            turn: 3,
        }
    );

    assert_eq!(
        game.play((2, 0)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), None, None],
                [None, Some(O), None],
                [Some(X), None, None]
            ],
            turn: 4,
        }
    );

    assert_eq!(
        game.play((1, 0)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), None, None],
                [Some(O), Some(O), None],
                [Some(X), None, None]
            ],
            turn: 5,
        }
    );

    assert_eq!(
        game.play((1, 2)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), None, None],
                [Some(O), Some(O), Some(X)],
                [Some(X), None, None]
            ],
            turn: 6,
        }
    );

    assert_eq!(
        game.play((0, 1)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), Some(O), None],
                [Some(O), Some(O), Some(X)],
                [Some(X), None, None]
            ],
            turn: 7,
        }
    );

    assert_eq!(
        game.play((2, 1)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), Some(O), None],
                [Some(O), Some(O), Some(X)],
                [Some(X), Some(X), None]
            ],
            turn: 8,
        }
    );

    assert_eq!(
        game.play((2, 2)),
        GameAccount {
            players: game.players(),
            state: Active,
            board: [
                [Some(X), Some(O), None],
                [Some(O), Some(O), Some(X)],
                [Some(X), Some(X), Some(O)]
            ],
            turn: 9,
        }
    );

    assert_eq!(
        game.play((0, 2)),
        GameAccount {
            players: game.players(),
            state: Tie,
            board: [
                [Some(X), Some(O), Some(X)],
                [Some(O), Some(O), Some(X)],
                [Some(X), Some(X), Some(O)]
            ],
            turn: 9,
        }
    );
}
