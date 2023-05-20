use std::fmt;

const SIZE: usize = 3;

#[derive(Clone, Copy, PartialEq)]
enum Cell {
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
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: [[Cell::Empty; SIZE]; SIZE],
            current_player: Cell::X,
        }
    }

    pub fn play(&mut self, x: usize, y: usize) -> bool {
        if self.board[x][y] != Cell::Empty {
            return false;
        }
        self.board[x][y] = self.current_player.clone();
        self.current_player = match self.current_player {
            Cell::X => Cell::O,
            Cell::O => Cell::X,
            _ => panic!("Invalid player"),
        };
        true
    }

    // method to create a game state string that can be sent to the client
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
    