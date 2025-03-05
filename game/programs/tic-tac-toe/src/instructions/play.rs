use anchor_lang::prelude::*;

use crate::{errors::TicTacToeError, state::game::*};

pub fn play(ctx: Context<Play>, tile: Tile) -> Result<()> {
    let game = &mut ctx.accounts.game;

    require!(game.is_active(), TicTacToeError::GameAlreadyOver);
    require_keys_eq!(
        game.current_player(),
        ctx.accounts.player.key(),
        TicTacToeError::NotPlayersTurn
    );

    game.play(&tile)
}

#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}
