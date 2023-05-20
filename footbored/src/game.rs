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

    // fn is_finished(&self) -> Option<Cell> {
    //     // Check rows and columns
    //     for i in 0..SIZE {
    //         if self.board[i][0] == self.board[i][1] && self.board[i][1] == self.board[i][2] && self.board[i][0] != Cell::Empty {
    //             return Some(self.board[i][0].clone());
    //         }
    //         if self.board[0][i] == self.board[1][i] && self.board[1][i] == self.board[2][i] && self.board[0][i] != Cell::Empty {
    //             return Some(self.board[0][i].clone());
    //         }
    //     }
    //     // Check diagonals
    //     if self.board[0][0] == self.board[1][1] && self.board[1][1] == self.board[2][2] && self.board[0][0] != Cell::Empty {
    //         return Some(self.board[0][0].clone());
    //     }
    //     if self.board[0][2] == self.board[1][1] && self.board[1][1] == self.board[2][0] && self.board[0][2] != Cell::Empty {
    //         return Some(self.board[0][2].clone());
    //     }
    //     // Check if there is any Empty cell left
    //     for i in 0..SIZE {
    //         for j in 0..SIZE {
    //             if self.board[i][j] == Cell::Empty {
    //                 return None;
    //             }
    //         }
    //     }
    //     // If all cells are full and no one has won, it's a draw
    //     Some(Cell::Empty)
    // }

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
    
    // fn main() {
    //     let mut game = Game::new();
    //     while let None = game.is_finished() {
    //         println!("{}", game);
    //         let mut input = String::new();
    //         io::stdin().read_line(&mut input).expect("Failed to read line");
    //         let coords: Vec<usize> = input.trim().split(',').map(|x| x.trim().parse().expect("Invalid input")).collect();
    //         if coords.len() != 2 {
    //             println!("Please enter coordinates in the format x,y");
    //             continue;
    //         }
    //         if !game.play(coords[0], coords[1]) {
    //             println!("Invalid move");
    //             continue;
    //         }
    //     }
    //     println!("{}", game);
    //     match game.is_finished() {
    //         Some(Cell::X) => println!("Player X wins!"),
    //         Some(Cell::O) => println!("Player O wins!"),
    //         _ => println!("It's a draw!"),
    //     }
    // }
    
