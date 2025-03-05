use anchor_lang::prelude::*;

use crate::errors::TicTacToeError;
type Board = [[Option<Sign>; 3]; 3];

#[account]
pub struct Game {
    players: [Pubkey; 2], // (32 * 2)
    turn: u8,             // 1
    board: Board,         // 9 * (1 + 1) = 18
    state: GameState,     // 32 + 1
}

impl Game {
    pub const MAXIMUM_SIZE: usize = (32 * 2) + 1 + (9 * (1 + 1)) + (32 + 1);

    pub fn start(&mut self, players: [Pubkey; 2]) -> Result<()> {
        // This next error can't happen, because SetupGame is the only
        // caller of `start`.
        require_eq!(self.turn, 0, TicTacToeError::GameAlreadyStarted);
        self.players = players;
        self.turn = 1;
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.state == GameState::Active
    }

    fn current_player_index(&self) -> usize {
        ((self.turn - 1) & 1) as usize
    }

    pub fn current_player(&self) -> Pubkey {
        self.players[self.current_player_index()]
    }

    pub fn play(&mut self, tile: &Tile) -> Result<()> {
        require!(self.is_active(), TicTacToeError::GameAlreadyOver);
        let (row, col) = (tile.0 as usize, tile.1 as usize);
        require!(row < 3 && col < 3, TicTacToeError::TileOutOfBounds);
        msg!(
            "Player {} plays at ({}, {}), old({:?})",
            self.current_player_index() + 1,
            row,
            col,
            self.board[row][col]
        );
        require!(
            self.board[row][col].is_none(),
            TicTacToeError::TileAlreadySet
        );
        self.board[row][col] = Some(Sign::from(self.current_player_index()));

        self.update_state();

        if GameState::Active == self.state {
            self.turn += 1;
        }

        Ok(())
    }

    fn is_winning_trio(&self, trio: [(usize, usize); 3]) -> bool {
        let [first, second, third] = trio;
        self.board[first.0][first.1].is_some()
            && self.board[first.0][first.1] == self.board[second.0][second.1]
            && self.board[first.0][first.1] == self.board[third.0][third.1]
    }

    fn update_state(&mut self) {
        if self.turn >= 5
            && (self.is_winning_trio([(0, 0), (0, 1), (0, 2)]) || // row 0
                self.is_winning_trio([(1, 0), (1, 1), (1, 2)]) || // row 1
                self.is_winning_trio([(2, 0), (2, 1), (2, 2)]) || // row 2
                self.is_winning_trio([(0, 0), (1, 0), (2, 0)]) || // column 0
                self.is_winning_trio([(0, 1), (1, 1), (2, 1)]) || // column 1
                self.is_winning_trio([(0, 2), (1, 2), (2, 2)]) || // column 2
                self.is_winning_trio([(0, 0), (1, 1), (2, 2)]) || // diagonal left to right
                self.is_winning_trio([(0, 2), (1, 1), (2, 0)] /*diagonal right to left*/))
        {
            // diagonal right to left
            self.state = GameState::Won {
                winner: self.current_player(),
            };
            return;
        }

        // maintain the state as Active if there have been less than 9 turns
        // and no one has won yet
        if self.turn >= 9 {
            self.state = GameState::Tie;
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sign {
    X,
    O,
}

impl From<usize> for Sign {
    fn from(value: usize) -> Self {
        match value {
            0 => Sign::X,
            1 => Sign::O,
            _ => panic!("Invalid value for Sign"),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Tile(u8, u8); // row, column
