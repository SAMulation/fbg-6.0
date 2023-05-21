use std::fmt;
use uuid::Uuid;

const SIZE: usize = 3;

#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    X,
    O,
}

// Implements the Display trait for Cell to allow for custom string formatting
impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Match the Cell variant and write the corresponding string representation to the formatter
        write!(f, "{}", match self {
            Cell::Empty => " ",
            Cell::X => "X",
            Cell::O => "O",
        })
    }
}

// Alias for the board type, a 2D array of Cells
type Board = [[Cell; SIZE]; SIZE];

pub struct Game {
    pub board: Board,
    pub current_player: Cell,
    pub players: Vec<Uuid>,
    pub winner: Option<Cell>,
    pub draw: bool,
}

impl Game {
    // Constructor method for Game
    pub fn new(player1_id: Uuid, player2_id: Uuid) -> Self {
        Game {
            board: [[Cell::Empty; SIZE]; SIZE],
            current_player: Cell::X,
            players: vec![player1_id, player2_id],
            winner: None,
            draw: false,
        }
    }

    // Method for playing a move in the game
    pub fn play(&mut self, player_id: Uuid, x: usize, y: usize) -> bool {
        // Check if it's the current player's turn
        if player_id != self.current_player_id() {
            return false;
        }

        // Check if the selected cell is already occupied
        if self.board[x][y] != Cell::Empty {
            return false;
        }

        // Update the board with the player's move
        self.board[x][y] = self.current_player;
        self.check_game_state(x, y);

        // Switch to the next player's turn
        self.current_player = match self.current_player {
            Cell::X => Cell::O,
            Cell::O => Cell::X,
            _ => panic!("Invalid player"),
        };
        true
    }

    // Returns the ID of the current player
    pub fn current_player_id(&self) -> Uuid {
        self.players[self.current_player as usize - 1]
    }

    // Checks the game state after each move to determine if there is a winner or a draw
    pub fn check_game_state(&mut self, x: usize, y: usize) {
        let mut row = 0;
        let mut col = 0;
        let mut diag1 = 0;
        let mut diag2 = 0;

        for i in 0..SIZE {
            if self.board[x][i] == self.current_player {
                row += 1;
            }
            if self.board[i][y] == self.current_player {
                col += 1;
            }
            if self.board[i][i] == self.current_player {
                diag1 += 1;
            }
            if self.board[i][SIZE - i - 1] == self.current_player {
                diag2 += 1;
            }
        }

        // Check if any row, column, or diagonal is filled with the current player's cells
        if row == SIZE || col == SIZE || diag1 == SIZE || diag2 == SIZE {
            self.winner = Some(self.current_player);
        } else if self.board.iter().flatten().all(|&cell| cell != Cell::Empty) {
            self.draw = true;
        }
    }

    // Converts the game state to a string representation
    pub fn to_string(&self) -> String {
        let mut state_string = String::new();
        for i in 0..SIZE {
            for j in 0..SIZE {
                state_string.push_str(&self.board[i][j].to_string());
                if j != SIZE - 1 {
                    state_string.push_str("|");
                }
            }
            if i != SIZE - 1 {
                state_string.push_str("\n");
            }
        }
        state_string
    }

    // Checks if the game is over (either a winner or a draw)
    pub fn is_over(&self) -> bool {
        self.winner.is_some() || self.draw
    }

    // Returns the winner of the game, if any
    pub fn get_winner(&self) -> Option<Cell> {
        self.winner
    }
}

// Implements the Display trait for Game to allow for custom string formatting
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..SIZE {
            if i != 0 {
                writeln!(f)?;
            }
            for j in 0..SIZE {
                if j != 0 {
                    write!(f, "|")?;
                }
                write!(f, "{}", self.board[i][j])?;
            }
        }
        Ok(())
    }
}
