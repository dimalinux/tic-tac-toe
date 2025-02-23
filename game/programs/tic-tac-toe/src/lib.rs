use anchor_lang::prelude::*;
use instructions::*;
use state::game::Tile;

pub mod errors;
pub mod instructions;
pub mod state;

// this key needs to be changed to whatever public key is returned by "anchor keys list"
declare_id!("6a9rKhtTGmCtKuxJCetCopbW5A5Z3A6hUsBq5mf1i3D");

#[program]
pub mod tic_tac_toe {
    use super::*;

    pub fn setup_game(ctx: Context<SetupGame>, player_two: Pubkey) -> Result<()> {
        setup_game::setup_game(ctx, player_two)
    }

    pub fn play(ctx: Context<Play>, tile: Tile) -> Result<()> {
        play::play(ctx, tile)
    }
}
