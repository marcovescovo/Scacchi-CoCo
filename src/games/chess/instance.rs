// Import necessary dependencies and modules
use super::super::util::Player;
use crate::game;
use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::Rng;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, DuplexStream, WriteHalf};
use tokio::time::{sleep_until, timeout, Duration, Instant};
use tracing::warn;

use super::color::Color;
use super::board::ChessBoard;
use super::chess_move::MoveType;
use std::io::{stdin, BufRead};

// Define a struct named 'Instance' to hold game-related parameters
#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) timeout: Duration,
    pub(crate) pace: Duration,
    pub(crate) rng: StdRng,
}

// Define a macro 'retired!' which sends a retirement message to players and spectators
macro_rules! retired {
    ($other:expr, $spectators:expr) => {{
        lnout2!($other, "RETIRE");
        lnout2!($spectators, "RETIRE");
        break;
    }};
}

// Function to refresh the player's turn color and print the corresponding message
pub fn refreshColor(turn: usize) -> Color {
    let mut current_color = Color::Black;
    if turn == 0 {
        current_color = Color::White;
        println!("White's turn");
    } else {
        println!("Black's turn");
    }
    current_color
}

// Implement the game::Instance trait for the defined Instance struct
#[async_trait]
impl game::Instance for Instance {
    // Define the 'start' method required by the trait
    async fn start(
        &mut self,
        players: HashMap<String, DuplexStream>,
        mut spectators: WriteHalf<DuplexStream>,
    ) {
        // Initialize the chess board and players
        let mut board = ChessBoard::new();
        let mut p = Player::from(players, &mut self.rng);
        assert_eq!(p.len(), 2);
        
        // Send player names to all participants
        for i in 0..2 {
            lnout2!(p[0].output, &p[i].name);
            lnout2!(p[1].output, &p[i].name);
            lnout2!(spectators, &p[i].name);
        }
        
        // Send initial messages to players
        lnout2!(p[0].output, "You have the white pieces");
        lnout2!(p[1].output, "You have the black pieces");
        
        let mut turn = 0;
        let mut current_color = Color::White;
        let mut retired = 0;
        let mut draw = 0;
        
        // Main game loop
        while !board.check_king_mate(current_color) && retired == 0 && draw != 2 {
            // Check if the current player's king is in check
            if board.check_king_check(current_color) {
                lnout2!(p[turn].output, "Your king is in check!");
                lnout2!(p[1 - turn].output, "You checked the opponent's king!");
                if turn == 0 {
                    lnout2!(spectators, "White king is checked!");
                } else {
                    lnout2!(spectators, "Black king is checked!");
                };
            }
            
            println!("\n");
            board.display();
            
            let start = Instant::now();
            
            // Read the player's move
            let mut buffer = String::new();
            let mut trimmed = String::new();
            
            // Use timeout to handle potential move input delays
            match timeout(self.timeout, p[turn].input.read_line(&mut buffer)).await {
                Ok(n) => {
                    trimmed = buffer.trim().to_string();
                }
                Err(err) => {
                    trimmed = buffer.trim().to_string();
                }
            };
            
            // Handle the draw condition
            if draw == 1 {
                if trimmed == "DRAW" {
                    lnout2!(p[turn].output, "Game ended: draw");
                    lnout2!(p[1 - turn].output, "Game ended: draw");
                    lnout2!(spectators, "Game ended: draw");
                    draw = draw + 1;
                } else {
                    draw = 0;
                    lnout2!(p[1 - turn].output, "Draw proposal refused");
                    lnout2!(spectators, "Draw proposal refused");
                    turn = 1 - turn;
                    current_color = refreshColor(turn);
                };
                continue;
            };
            
            // Process the player's move
            let opt = MoveType::parse(&trimmed);
            if !board.check_move(opt, current_color) {
                if trimmed == "RETIRE" {
                    retired = 1;
                    retired!(p[1 - turn].output, spectators)
                } else {
                    if trimmed == "DRAW" {
                        draw = 1;
                        lnout2!(p[turn].output, "Draw proposed");
                        lnout2!(p[1 - turn].output, "Draw proposed");
                        lnout2!(spectators, "Draw proposed");
                        turn = 1 - turn;
                        current_color = refreshColor(turn);
                    } else {
                        lnout2!(p[turn].output, "Invalid move");
                    }
                };
                continue;
            } else {
                lnout2!(p[1 - turn].output, &trimmed);
                lnout2!(spectators, &trimmed);
                
                board = board.apply_move_type(opt.unwrap());
                turn = 1 - turn;
                current_color = refreshColor(turn);
                sleep_until(start + self.pace).await;
                continue;
            }
        }
        
        // Game ending messages
        lnout2!(p[1 - turn].output, "CHECKMATE! You win!");
        lnout2!(p[turn].output, "CHECKMATE! You loose!");
        if turn == 0 {
            lnout2!(spectators, "CHECKMATE! Black wins!");
        } else {
            lnout2!(spectators, "CHECKMATE! White wins!");
        };
    }
}
