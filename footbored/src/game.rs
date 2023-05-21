use std::fmt;

const SIZE: usize = 3;

#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    X,
    O,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Cell::Empty => " ",
            Cell::X => "X",
            Cell::O => "O",
        })
    }
}

type Board = [[Cell; SIZE]; SIZE];

pub struct Game {
    board: Board,
    current_player: Cell,
    winner: Option<Cell>,
    draw: bool,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: [[Cell::Empty; SIZE]; SIZE],
            current_player: Cell::X,
            winner: None,
            draw: false,
        }
    }

    pub fn play(&mut self, x: usize, y: usize) -> bool {
        if self.board[x][y] != Cell::Empty {
            return false;
        }
        self.board[x][y] = self.current_player;
        self.check_game_state(x, y);
        self.current_player = match self.current_player {
            Cell::X => Cell::O,
            Cell::O => Cell::X,
            _ => panic!("Invalid player"),
        };
        true
    }

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

        if row == SIZE || col == SIZE || diag1 == SIZE || diag2 == SIZE {
            self.winner = Some(self.current_player);
        } else if self.board.iter().flatten().all(|&cell| cell != Cell::Empty) {
            self.draw = true;
        }
    }

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

    pub fn is_over(&self) -> bool {
        self.winner.is_some() || self.draw
    }

    pub fn get_winner(&self) -> Option<Cell> {
        self.winner
    }
}

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
