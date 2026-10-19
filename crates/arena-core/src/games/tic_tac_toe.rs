use crate::game::{Error, Event, Game, Outcome, Player, Seed, Turn};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const PLAYERS: usize = 2;
const BOARD_WIDTH: usize = 3;
const BOARD_SIZE: usize = BOARD_WIDTH * BOARD_WIDTH;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = TicTacToeAction)]
pub struct Action {
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicTacToe {
    board: [Option<Player>; BOARD_SIZE],
    turn: Turn,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[schema(as = TicTacToeView)]
pub struct View {
    pub board: [Option<Player>; BOARD_SIZE],
}

const WINNING_LINES: [[usize; BOARD_WIDTH]; 8] = [
    // Rows.
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    // Columns.
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    // Diagonals.
    [0, 4, 8],
    [2, 4, 6],
];

impl TicTacToe {
    fn has_winning_line(&self, player: Player) -> bool {
        WINNING_LINES
            .iter()
            .any(|line| line.iter().all(|&index| self.board[index] == Some(player)))
    }
}

impl Game for TicTacToe {
    type Config = crate::game::GameConfig;
    type Action = Action;
    type Event = Event;
    type View = View;

    fn with_config(players: usize, _: Seed, _: Self::Config) -> Result<Self, Error> {
        if players != PLAYERS {
            return Err(Error::PlayerCount);
        }
        Ok(Self {
            board: [None; BOARD_SIZE],
            turn: Turn::first(),
            outcome: None,
        })
    }

    fn turn(&self) -> Turn {
        self.turn.clone()
    }

    fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }

    fn legal_actions(&self, player: Player) -> Vec<Action> {
        if self.outcome.is_some() || !self.turn.players.contains(&player) {
            return vec![];
        }
        self.board
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_none())
            .map(|(index, _)| Action { index })
            .collect()
    }

    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        self.board[action.index] = Some(player);
        if self.has_winning_line(player) {
            self.outcome = Some(Outcome {
                winners: vec![player],
                scores: None,
            });
        } else if self.board.iter().all(Option::is_some) {
            self.outcome = Some(Outcome {
                winners: vec![],
                scores: None,
            });
        }
        self.turn.advance(PLAYERS);
        vec![Event::Played { player }]
    }

    fn view(&self, _: Player) -> View {
        View { board: self.board }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;
    use std::collections::HashSet;

    fn play(indices: &[usize]) -> Session<TicTacToe> {
        let mut game = Session::<TicTacToe>::new(2, [0; 32]).unwrap();
        for &index in indices {
            game.act(game.state().turn().players[0], Action { index })
                .unwrap();
        }
        game
    }

    #[test]
    fn both_players_can_win_and_a_last_cell_win_is_not_a_draw() {
        for (moves, winner) in [
            (vec![0, 3, 1, 4, 2], 0),
            (vec![0, 3, 1, 4, 8, 5], 1),
            (vec![0, 1, 2, 4, 3, 5, 7, 8, 6], 0),
        ] {
            let game = play(&moves);
            let outcome = Outcome {
                winners: vec![winner],
                scores: None,
            };
            assert_eq!(game.state().outcome(), Some(outcome.clone()));
            assert_eq!(
                game.records().last().unwrap().events.last(),
                Some(&Event::Finished { outcome })
            );
        }
    }

    #[test]
    fn a_full_board_without_a_line_is_a_draw() {
        let game = play(&[0, 1, 2, 4, 3, 5, 7, 6, 8]);
        assert_eq!(
            game.state().outcome(),
            Some(Outcome {
                winners: vec![],
                scores: None
            })
        );
        assert!(game.state().legal_actions(0).is_empty());
    }

    #[test]
    fn rejections_preserve_state_and_history() {
        let mut game = play(&[]);
        let before = serde_json::to_value(&game).unwrap();
        assert_eq!(
            game.act(1, Action { index: 0 }).unwrap_err(),
            Error::NotYourTurn
        );
        for index in [9, usize::MAX] {
            assert_eq!(
                game.act(0, Action { index }).unwrap_err(),
                Error::IllegalMove
            );
        }
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
        game.act(0, Action { index: 0 }).unwrap();
        let before = serde_json::to_value(&game).unwrap();
        assert_eq!(
            game.act(1, Action { index: 0 }).unwrap_err(),
            Error::IllegalMove
        );
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
        let mut game = play(&[0, 3, 1, 4, 2]);
        let before = serde_json::to_value(&game).unwrap();
        assert_eq!(
            game.act(1, Action { index: 5 }).unwrap_err(),
            Error::Finished
        );
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
    }

    #[test]
    fn all_reachable_positions_have_correct_results_and_legal_actions() {
        fn visit(game: Session<TicTacToe>, seen: &mut HashSet<[Option<Player>; BOARD_SIZE]>) {
            let state = game.state();
            let board = state.view(0).board;
            if !seen.insert(board) {
                return;
            }
            assert_eq!(state.view(0), state.view(1));
            let filled = board.iter().flatten().count();
            assert_eq!(
                state.turn(),
                Turn {
                    players: vec![filled % PLAYERS],
                    number: filled as u64
                }
            );
            // Check by coordinates, independently of the game's winning-line table.
            let winner = (0..PLAYERS).find(|&player| {
                let owns = |row, column| board[row * BOARD_WIDTH + column] == Some(player);
                let row_win =
                    (0..BOARD_WIDTH).any(|row| (0..BOARD_WIDTH).all(|column| owns(row, column)));
                let column_win =
                    (0..BOARD_WIDTH).any(|column| (0..BOARD_WIDTH).all(|row| owns(row, column)));
                let diagonal_win = (0..BOARD_WIDTH).all(|index| owns(index, index))
                    || (0..BOARD_WIDTH).all(|index| owns(index, BOARD_WIDTH - 1 - index));
                row_win || column_win || diagonal_win
            });
            let expected = if let Some(player) = winner {
                Some(Outcome {
                    winners: vec![player],
                    scores: None,
                })
            } else if filled == BOARD_SIZE {
                Some(Outcome {
                    winners: vec![],
                    scores: None,
                })
            } else {
                None
            };
            assert_eq!(state.outcome(), expected);
            let legal = state.legal_actions(state.turn().players[0]);
            let indices: Vec<_> = if expected.is_some() {
                vec![]
            } else {
                (0..BOARD_SIZE)
                    .filter(|&index| board[index].is_none())
                    .collect()
            };
            assert_eq!(legal.iter().map(|a| a.index).collect::<Vec<_>>(), indices);
            for action in legal {
                let mut next = game.clone();
                next.act(state.turn().players[0], action).unwrap();
                visit(next, seen);
            }
        }
        let mut seen = HashSet::new();
        visit(play(&[]), &mut seen);
        assert_eq!(seen.len(), 5478);
    }
}
